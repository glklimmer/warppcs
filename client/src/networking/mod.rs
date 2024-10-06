use bevy::prelude::*;

use bevy::sprite::Mesh2dHandle;
use bevy_renet::{
    client_connected,
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
        ClientId, RenetClient,
    },
    RenetClientPlugin,
};
use shared::{
    map::base::MainBuildingBundle,
    map::GameSceneType,
    networking::{
        connection_config, ClientChannel, NetworkedEntities, PlayerCommand, PlayerInput, Rotation,
        ServerChannel, ServerMessages, SpawnPlayer, SpawnProjectile, SpawnUnit, PROTOCOL_ID,
    },
};
use spawn::{spawn_player, spawn_projectile, spawn_unit};
use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

mod spawn;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

#[derive(Debug, Default, Resource)]
pub struct ClientLobby {
    pub players: HashMap<ClientId, PlayerEntityMapping>,
}

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Debug, Resource)]
pub struct CurrentClientId(pub u64);

#[derive(Debug)]
pub struct PlayerEntityMapping {
    client_entity: Entity,
    server_entity: Entity,
}

#[derive(Default, Resource)]
struct NetworkMapping(HashMap<Entity, Entity>);

pub enum Change {
    Rotation(Rotation),
    Movement(bool),
    Attack,
}

#[derive(Event)]
pub struct NetworkEvent {
    pub entity: Entity,
    pub change: Change,
}

#[derive(Component)]
struct PartOfScene;

pub struct ClientNetworkingPlugin;

impl Plugin for ClientNetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);

        add_netcode_network(app);

        app.insert_resource(NetworkMapping::default());
        app.insert_resource(ClientLobby::default());

        app.add_event::<NetworkEvent>();
        app.add_event::<SpawnPlayer>();
        app.add_event::<SpawnUnit>();
        app.add_event::<SpawnProjectile>();

        app.add_systems(
            Update,
            (
                client_sync_players,
                client_send_input,
                client_send_player_commands,
                spawn_player,
                spawn_unit,
                spawn_projectile,
            )
                .in_set(Connected),
        );
    }
}

fn add_netcode_network(app: &mut App) {
    app.add_plugins(bevy_renet::transport::NetcodeClientPlugin);

    app.configure_sets(Update, Connected.run_if(client_connected));

    let client = RenetClient::new(connection_config());

    let server_addr = "127.0.0.1:6969".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    app.insert_resource(client);
    app.insert_resource(transport);
    app.insert_resource(CurrentClientId(client_id));

    // If any error is found we just panic
    #[allow(clippy::never_loop)]
    fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
        for e in renet_error.read() {
            panic!("{}", e);
        }
    }

    app.add_systems(Update, panic_on_error_system);
}

#[allow(clippy::too_many_arguments)]
fn client_sync_players(
    mut commands: Commands,
    mut transforms: Query<&mut Transform>,
    entities: Query<Entity, With<PartOfScene>>,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut network_events: EventWriter<NetworkEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut spawn_player: EventWriter<SpawnPlayer>,
    mut spawn_unit: EventWriter<SpawnUnit>,
    mut spawn_projectile: EventWriter<SpawnProjectile>,
) {
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::SpawnPlayer(spawn) => {
                spawn_player.send(spawn);
            }
            ServerMessages::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                if let Some(PlayerEntityMapping {
                    server_entity,
                    client_entity,
                }) = lobby.players.remove(&id)
                {
                    commands.entity(client_entity).despawn();
                    network_mapping.0.remove(&server_entity);
                }
            }
            ServerMessages::MeleeAttack {
                entity: server_entity,
            } => {
                if let Some(client_entity) = network_mapping.0.get(&server_entity) {
                    network_events.send(NetworkEvent {
                        entity: *client_entity,
                        change: Change::Attack,
                    });
                }
            }
            ServerMessages::SpawnUnit(spawn) => {
                spawn_unit.send(spawn);
            }
            ServerMessages::DespawnEntity {
                entity: server_entity,
            } => {
                if let Some(client_entity) = network_mapping.0.remove(&server_entity) {
                    if let Some(mut entity) = commands.get_entity(client_entity) {
                        entity.despawn();
                    }
                }
            }
            ServerMessages::SpawnProjectile(spawn) => {
                spawn_projectile.send(spawn);
            }
            ServerMessages::LoadGameScene {
                game_scene_type: map_type,
                players,
                units,
                projectiles,
            } => {
                println!("Loading map {:?}...", map_type);

                for entity in entities.iter() {
                    commands.entity(entity).despawn();
                }

                match map_type {
                    GameSceneType::Base(color) => {
                        commands.spawn((
                            MainBuildingBundle::new(),
                            (
                                Mesh2dHandle(meshes.add(Rectangle::new(200., 100.))),
                                materials.add(color),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ));
                    }
                    GameSceneType::Camp => todo!(),
                };
                println!("revieved {} players", players.len());
                players.into_iter().for_each(|spawn| {
                    spawn_player.send(spawn);
                });
                units.into_iter().for_each(|spawn| {
                    spawn_unit.send(spawn);
                });
                projectiles.into_iter().for_each(|spawn| {
                    spawn_projectile.send(spawn);
                });
            }
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let maybe_net_entities: Result<NetworkedEntities, _> = bincode::deserialize(&message);
        match maybe_net_entities {
            Ok(networked_entities) => {
                for i in 0..networked_entities.entities.len() {
                    if let Some(client_entity) = network_mapping
                        .0
                        .get(&networked_entities.entities[i].entity)
                    {
                        let network_entity = &networked_entities.entities[i];

                        if let Ok(mut transform) = transforms.get_mut(*client_entity) {
                            transform.translation = network_entity.translation.into();
                        }

                        network_events.send(NetworkEvent {
                            entity: *client_entity,
                            change: Change::Rotation(network_entity.rotation.clone()),
                        });

                        network_events.send(NetworkEvent {
                            entity: *client_entity,
                            change: Change::Movement(network_entity.moving),
                        });
                    }
                }
            }
            Err(error) => error!("Error on deserialize: {}", error),
        }
    }
}

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();
    client.send_message(ClientChannel::Input, input_message);
}

fn client_send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.read() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command, command_message);
    }
}
