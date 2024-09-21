use bevy::prelude::*;

use crate::{
    server::{
        ai::{
            attack::{unit_health, unit_swing_timer},
            UnitBehaviour,
        },
        physics::{collider::BoxCollider, movement::Velocity},
    },
    shared::networking::{
        ClientChannel, Facing, NetworkEntity, NetworkedEntities, Owner, PlayerCommand, PlayerInput,
        ProjectileType, Rotation, ServerChannel, ServerMessages, Unit,
    },
};
use bevy_renet::{
    renet::{ClientId, RenetServer, ServerEvent},
    RenetServerPlugin,
};

use crate::shared::networking::{connection_config, PROTOCOL_ID};
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::transport::NetcodeServerPlugin;
use std::collections::HashMap;
use std::{net::UdpSocket, time::SystemTime};

#[derive(Debug, Component)]
struct ServerPlayer {
    id: ClientId,
}

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<ClientId, Entity>,
}

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (server_update_system, server_network_sync, on_unit_death),
        );

        app.add_plugins(RenetServerPlugin);
        app.insert_resource(ServerLobby::default());

        add_netcode_network(app);
    }
}

fn add_netcode_network(app: &mut App) {
    app.add_plugins(NetcodeServerPlugin);

    let server = RenetServer::new(connection_config());

    let public_addr = "127.0.0.1:6969".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time: std::time::Duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    app.insert_resource(server);
    app.insert_resource(transport);
}

fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    players: Query<(Entity, &ServerPlayer, &Transform)>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {} connected.", client_id);

                // Initialize other players for this new client
                for (entity, player, transform) in players.iter() {
                    let translation: [f32; 3] = transform.translation.into();
                    let message = bincode::serialize(&ServerMessages::PlayerCreate {
                        id: player.id,
                        entity,
                        translation,
                    })
                    .unwrap();
                    server.send_message(*client_id, ServerChannel::ServerMessages, message);
                }

                // Spawn new player
                let transform = Transform::from_xyz(
                    (fastrand::f32() - 0.5) * 200.,
                    50.0,
                    (fastrand::f32() - 0.5) * 200.,
                );

                let player_entity = commands
                    .spawn((
                        transform,
                        PlayerInput::default(),
                        Velocity::default(),
                        ServerPlayer { id: *client_id },
                    ))
                    .id();

                lobby.players.insert(*client_id, player_entity);

                let translation: [f32; 3] = transform.translation.into();
                let message = bincode::serialize(&ServerMessages::PlayerCreate {
                    id: *client_id,
                    entity: player_entity,
                    translation,
                })
                .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Player {} disconnected: {}", client_id, reason);

                if let Some(player_entity) = lobby.players.remove(client_id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerRemove { id: *client_id }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
            let command: PlayerCommand = bincode::deserialize(&message).unwrap();
            match command {
                PlayerCommand::MeleeAttack => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        let message = ServerMessages::MeleeAttack {
                            entity: *player_entity,
                        };
                        let message = bincode::serialize(&message).unwrap();
                        server.broadcast_message(ServerChannel::ServerMessages, message);
                    }
                }
                PlayerCommand::SpawnUnit(unit_type) => {
                    println!(
                        "Received spawn unit from client {}: {:?}",
                        client_id, unit_type
                    );

                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        if let Ok((_, _, player_transform)) = players.get(*player_entity) {
                            let unit = Unit {
                                health: unit_health(&unit_type),
                                swing_timer: unit_swing_timer(&unit_type),
                                unit_type: unit_type.clone(),
                            };

                            let unit_entity = commands
                                .spawn((
                                    Transform::from_translation(player_transform.translation),
                                    unit,
                                    Owner(client_id),
                                    Velocity::default(),
                                    UnitBehaviour::Idle,
                                    BoxCollider(Vec2::new(50., 90.)),
                                ))
                                .id();

                            let message = ServerMessages::SpawnUnit {
                                entity: unit_entity,
                                owner: Owner(client_id),
                                unit_type,
                                translation: player_transform.translation.into(),
                            };
                            let message = bincode::serialize(&message).unwrap();
                            server.broadcast_message(ServerChannel::ServerMessages, message);
                        }
                    }
                }
            }
        }
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let input: PlayerInput = bincode::deserialize(&message).unwrap();
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(input);
            }
        }
    }
}

fn server_network_sync(
    mut server: ResMut<RenetServer>,
    unit_query: Query<(Entity, &Transform, &Velocity), Without<ProjectileType>>,
    projectile_query: Query<(Entity, &Transform, &Velocity), With<ProjectileType>>,
) {
    let mut networked_entities = NetworkedEntities::default();
    for (entity, transform, velocity) in unit_query.iter() {
        let movement = Rotation::LeftRight {
            facing: match velocity.0.x.total_cmp(&0.) {
                std::cmp::Ordering::Less => Some(Facing::Left),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(Facing::Right),
            },
        };

        networked_entities.entities.push(NetworkEntity {
            entity,
            translation: transform.translation.into(),
            rotation: movement,
            moving: velocity.0.x != 0.,
        });
    }

    for (entity, transform, velocity) in projectile_query.iter() {
        if velocity.0.x == 0. && velocity.0.y == 0. {
            continue;
        }

        let orientation = Rotation::Free {
            angle: (velocity.0.to_angle()),
        };

        networked_entities.entities.push(NetworkEntity {
            entity,
            translation: transform.translation.into(),
            rotation: orientation,
            moving: true,
        });
    }

    let sync_message = bincode::serialize(&networked_entities).unwrap();
    server.broadcast_message(ServerChannel::NetworkedEntities, sync_message);
}

fn on_unit_death(
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
    query: Query<(Entity, &Unit)>,
) {
    for (entity, unit) in query.iter() {
        if unit.health <= 0. {
            commands.entity(entity).despawn_recursive();

            let message = ServerMessages::DespawnEntity { entity };
            let unit_dead_message = bincode::serialize(&message).unwrap();
            server.broadcast_message(ServerChannel::ServerMessages, unit_dead_message);
        }
    }
}
