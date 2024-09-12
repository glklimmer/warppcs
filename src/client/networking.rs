use bevy::prelude::*;

use crate::{
    client::king::{AnimationsState, PaladinBundle, WarriorBundle},
    shared::networking::{
        connection_config, ClientChannel, NetworkedEntities, PlayerCommand, PlayerInput,
        ServerChannel, ServerMessages, UnitType, PROTOCOL_ID,
    },
};
use bevy_renet::{
    client_connected,
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
        ClientId, RenetClient,
    },
    RenetClientPlugin,
};
use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

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

#[derive(Event)]
pub enum UnitEvent {
    MeleeAttack(Entity),
}

pub struct ClientNetworkingPlugin;

impl Plugin for ClientNetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);

        add_netcode_network(app);

        app.add_event::<UnitEvent>();

        app.insert_resource(NetworkMapping::default());
        app.insert_resource(ClientLobby::default());

        app.add_systems(
            Update,
            (
                client_sync_players,
                client_send_input,
                client_send_player_commands,
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
fn client_sync_players(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    client_id: Res<CurrentClientId>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut unit_events: EventWriter<UnitEvent>,
    warrior_sprite_sheet: Res<WarriorSpriteSheet>,
    paladin_sprite_sheet: Res<PaladinSpriteSheet>,
) {
    let client_id = client_id.0;
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerCreate {
                id,
                translation,
                entity,
            } => {
                println!("Player {} connected.", id);

                let mut client_entity = match id.raw() {
                    1 => commands.spawn(PaladinBundle::new(
                        &paladin_sprite_sheet,
                        translation,
                        AnimationsState::Idle,
                    )),
                    _ => commands.spawn(WarriorBundle::new(
                        &warrior_sprite_sheet,
                        translation,
                        AnimationsState::Idle,
                    )),
                };

                if client_id == id.raw() {
                    client_entity.insert(ControlledPlayer);
                }

                let player_info = PlayerEntityMapping {
                    server_entity: entity,
                    client_entity: client_entity.id(),
                };

                lobby.players.insert(id, player_info);
                network_mapping.0.insert(entity, client_entity.id());
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
            ServerMessages::MeleeAttack { entity } => {
                if let Some(entity) = network_mapping.0.get(&entity) {
                    unit_events.send(UnitEvent::MeleeAttack(*entity));
                }
            }
            ServerMessages::SpawnUnit {
                entity,
                owner,
                unit_type,
                translation,
            } => {
                println!("Spawning Unit for player {} connected.", owner);
                let texture = match unit_type {
                    UnitType::Warrior => asset_server.load("aseprite/warrior.png"),
                    UnitType::Pikeman => asset_server.load("aseprite/pike_man.png"),
                    UnitType::Archer => asset_server.load("aseprite/archer.png"),
                };

                let unit_entity = commands
                    .spawn(SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(translation[0], translation[1], translation[2]),
                            scale: Vec3::splat(3.0),
                            ..Default::default()
                        },
                        texture,
                        ..default()
                    })
                    .id();

                network_mapping.0.insert(entity, unit_entity);
            }
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();

        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping
                .0
                .get(&networked_entities.entities[i].entity)
            {
                let network_entity = &networked_entities.entities[i];
                let movement = network_entity.movement.clone();

                commands.entity(*entity).insert(movement);
            }
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
