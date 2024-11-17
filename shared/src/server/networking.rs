use bevy::prelude::*;

use crate::map::{GameScene, GameSceneId};
use crate::networking::{
    ClientChannel, Facing, MultiplayerRoles, NetworkEntity, NetworkedEntities, PlayerCommand,
    PlayerInput, ProjectileType, Rotation, ServerChannel, ServerMessages, Unit,
};
use crate::server::economy::GoldAmount;
use crate::server::game_scenes::GameSceneDestination;
use crate::server::physics::movement::Velocity;
use crate::{BoxCollider, GameState};

use bevy_renet::{
    renet::{ClientId, RenetServer, ServerEvent},
    RenetServerPlugin,
};
use std::collections::HashMap;

use super::ai::AIPlugin;
use super::buildings::BuildingsPlugins;
use super::economy::EconomyPlugin;
use super::game_scenes::GameScenesPlugin;
use super::lobby::{LobbyPlugin, PlayerJoinedLobby, PlayerLeftLobby};
use super::physics::PhysicsPlugin;
use super::players::PlayerPlugin;

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<ClientId, Entity>,
}

#[derive(Default, Resource)]
pub struct GameWorld {
    pub game_scenes: HashMap<GameSceneId, GameScene>,
}

#[derive(Component)]
pub struct ServerPlayer(pub ClientId);

#[derive(Event)]
pub struct NetworkEvent {
    pub client_id: ClientId,
    pub message: PlayerCommand,
}

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NetworkEvent>();

        app.add_plugins(AIPlugin);
        app.add_plugins(PhysicsPlugin);
        app.add_plugins(GameScenesPlugin);
        app.add_plugins(BuildingsPlugins);
        app.add_plugins(EconomyPlugin);

        app.add_systems(
            FixedUpdate,
            (
                receive_client_messages,
                sync_networked_entities,
                client_connections,
            )
                .run_if(in_state(MultiplayerRoles::Host)),
        );

        app.add_systems(
            FixedUpdate,
            (on_unit_death).run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );

        app.insert_resource(ServerLobby::default());
        app.add_plugins(RenetServerPlugin);
        app.add_plugins(LobbyPlugin);

        app.insert_resource(GameWorld::default());
    }
}

fn client_connections(
    mut commands: Commands,
    mut server_events: EventReader<ServerEvent>,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    mut player_joined: EventWriter<PlayerJoinedLobby>,
    mut player_left: EventWriter<PlayerLeftLobby>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {} connected.", client_id);

                let player_entity = commands
                    .spawn((ServerPlayer(*client_id), BoxCollider(Vec2::new(50., 90.))))
                    .id();

                lobby.players.insert(*client_id, player_entity);

                player_joined.send(PlayerJoinedLobby(*client_id));
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Player {} disconnected: {}", client_id, reason);

                if let Some(player_entity) = lobby.players.remove(client_id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerDisconnected { id: *client_id })
                        .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);

                player_left.send(PlayerLeftLobby(*client_id));
            }
        }
    }
}

fn receive_client_messages(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut player_commands: EventWriter<NetworkEvent>,
    lobby: Res<ServerLobby>,
) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
            let command: PlayerCommand = bincode::deserialize(&message).unwrap();
            player_commands.send(NetworkEvent {
                client_id,
                message: command,
            });
        }
    }

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let input: PlayerInput = bincode::deserialize(&message).unwrap();
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(input);
            }
        }
    }
}

fn sync_networked_entities(
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

            let message = ServerMessages::DespawnEntity {
                entities: vec![entity],
            };
            let unit_dead_message = bincode::serialize(&message).unwrap();
            server.broadcast_message(ServerChannel::ServerMessages, unit_dead_message);
        }
    }
}
