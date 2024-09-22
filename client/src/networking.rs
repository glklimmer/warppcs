use bevy::prelude::*;

use bevy_renet::{
    client_connected,
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
        ClientId, RenetClient,
    },
    RenetClientPlugin,
};
use shared::{
    networking::{
        connection_config, ClientChannel, NetworkedEntities, PlayerCommand, PlayerInput,
        ProjectileType, Rotation, ServerChannel, ServerMessages, UnitType, PROTOCOL_ID,
    },
    BoxCollider,
};
use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

use crate::{
    animation::UnitAnimation,
    king::{PaladinBundle, WarriorBundle},
};

use super::king::{PaladinSpriteSheet, WarriorSpriteSheet};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

#[derive(Debug, Default, Resource)]
struct ClientLobby {
    players: HashMap<ClientId, PlayerEntityMapping>,
}

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Debug, Resource)]
struct CurrentClientId(u64);

#[derive(Debug)]
struct PlayerEntityMapping {
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
struct Despawning;

pub struct ClientNetworkingPlugin;

impl Plugin for ClientNetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);

        add_netcode_network(app);

        app.insert_resource(NetworkMapping::default());
        app.insert_resource(ClientLobby::default());

        app.add_event::<NetworkEvent>();

        app.add_systems(
            Update,
            (
                client_sync_players,
                client_send_input,
                client_send_player_commands,
            )
                .in_set(Connected),
        );

        app.add_systems(PostUpdate, (despawn_entities,).in_set(Connected));
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
    mut client: ResMut<RenetClient>,
    client_id: Res<CurrentClientId>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut network_events: EventWriter<NetworkEvent>,
    warrior_sprite_sheet: Res<WarriorSpriteSheet>,
    paladin_sprite_sheet: Res<PaladinSpriteSheet>,
    asset_server: Res<AssetServer>,
) {
    let client_id = client_id.0;
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerCreate {
                id,
                translation,
                entity: server_player_entity,
            } => {
                println!("Player {} connected.", id);

                let mut client_player_entity = match lobby.players.len() {
                    0 => commands.spawn(PaladinBundle::new(
                        &paladin_sprite_sheet,
                        translation,
                        UnitAnimation::Idle,
                    )),
                    _ => commands.spawn(WarriorBundle::new(
                        &warrior_sprite_sheet,
                        translation,
                        UnitAnimation::Idle,
                    )),
                };

                if client_id == id.raw() {
                    client_player_entity
                        .insert((ControlledPlayer, BoxCollider(Vec2::new(50., 90.))));
                }

                let player_info = PlayerEntityMapping {
                    server_entity: server_player_entity,
                    client_entity: client_player_entity.id(),
                };

                lobby.players.insert(id, player_info);
                network_mapping
                    .0
                    .insert(server_player_entity, client_player_entity.id());
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
            ServerMessages::SpawnUnit {
                entity: server_unit_entity,
                owner,
                translation,
                unit_type,
            } => {
                let texture = match unit_type {
                    UnitType::Shieldwarrior => asset_server.load("aseprite/shield_warrior.png"),
                    UnitType::Pikeman => asset_server.load("aseprite/pike_man.png"),
                    UnitType::Archer => asset_server.load("aseprite/archer.png"),
                };

                let client_unit_entity = commands
                    .spawn((
                        SpriteBundle {
                            transform: Transform {
                                translation: translation.into(),
                                scale: Vec3::splat(3.0),
                                ..default()
                            },
                            texture,
                            ..default()
                        },
                        owner,
                    ))
                    .id();

                network_mapping
                    .0
                    .insert(server_unit_entity, client_unit_entity);
            }
            ServerMessages::DespawnEntity {
                entity: server_entity,
            } => {
                if let Some(client_entity) = network_mapping.0.remove(&server_entity) {
                    commands.entity(client_entity).insert(Despawning);
                }
            }
            ServerMessages::SpawnProjectile {
                entity: server_entity,
                projectile_type,
                translation,
                direction,
            } => {
                let texture = match projectile_type {
                    ProjectileType::Arrow => asset_server.load("aseprite/arrow.png"),
                };

                let direction: Vec2 = direction.into();
                let position: Vec3 = translation.into();
                let position = position.truncate();

                let angle = (direction - position).angle_between(position);

                let client_entity = commands
                    .spawn((SpriteBundle {
                        transform: Transform {
                            translation: translation.into(),
                            scale: Vec3::splat(2.0),
                            rotation: Quat::from_rotation_z(angle),
                        },
                        texture,
                        ..default()
                    },))
                    .id();

                network_mapping.0.insert(server_entity, client_entity);
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

fn despawn_entities(mut commands: Commands, query: Query<Entity, With<Despawning>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
