use bevy::prelude::*;

use crate::map::base::BaseScene;
use crate::map::{GameScene, GameSceneId, GameSceneType, Layers};
use crate::networking::{
    ClientChannel, Facing, MultiplayerRoles, NetworkEntity, NetworkedEntities, Owner,
    PlayerCommand, PlayerInput, PlayerSkin, ProjectileType, Rotation, ServerChannel,
    ServerMessages, SpawnPlayer, Unit,
};
use crate::server::game_scenes::GameSceneDestination;
use crate::server::physics::movement::Velocity;
use crate::{BoxCollider, GameState};

use bevy::color::palettes::css::{BLUE, RED};
use bevy_renet::{
    renet::{ClientId, RenetServer, ServerEvent},
    RenetServerPlugin,
};
use std::collections::HashMap;

use super::ai::AIPlugin;
use super::buildings::BuildingsPlugins;
use super::game_scenes::GameScenesPlugin;
use super::lobby::{
    GameLobby, LobbyPlugin, PlayerChangedReady, PlayerJoinedLobby, PlayerLeavedLobby,
};
use super::physics::PhysicsPlugin;

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
pub struct InteractEvent(pub ClientId);

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractEvent>();

        app.add_plugins(AIPlugin);
        app.add_plugins(PhysicsPlugin);
        app.add_plugins(GameScenesPlugin);
        app.add_plugins(BuildingsPlugins);

        app.add_systems(
            FixedUpdate,
            (
                server_update_system,
                server_network_sync,
                server_lobby_system,
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

fn server_lobby_system(
    mut commands: Commands,
    mut server_events: EventReader<ServerEvent>,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    mut player_joined: EventWriter<PlayerJoinedLobby>,
    mut player_left: EventWriter<PlayerLeavedLobby>,
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

                player_left.send(PlayerLeavedLobby(*client_id));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn server_update_system(
    mut commands: Commands,
    lobby: Res<ServerLobby>,
    mut server: ResMut<RenetServer>,
    mut game_world: ResMut<GameWorld>,
    mut interact: EventWriter<InteractEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_checkbox: EventWriter<PlayerChangedReady>,
    game_lobby: Res<GameLobby>,
) {
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
                PlayerCommand::StartGame => {
                    #[cfg(prod)]
                    if !game_lobby.all_ready() {
                        continue;
                    }
                    println!("Starting game...");
                    for (client_id, player_entity) in lobby.players.iter() {
                        let (game_scene_id, skin, color, left_destination, right_destination) =
                            if game_world.game_scenes.is_empty() {
                                (
                                    GameSceneId(1),
                                    PlayerSkin::Warrior,
                                    BLUE,
                                    GameSceneDestination {
                                        scene: GameSceneId(2),
                                        position: Vec3::new(-1300., 50., Layers::Player.as_f32()),
                                    },
                                    GameSceneDestination {
                                        scene: GameSceneId(2),
                                        position: Vec3::new(1300., 50., Layers::Player.as_f32()),
                                    },
                                )
                            } else {
                                (
                                    GameSceneId(2),
                                    PlayerSkin::Monster,
                                    RED,
                                    GameSceneDestination {
                                        scene: GameSceneId(1),
                                        position: Vec3::new(-1300., 50., Layers::Player.as_f32()),
                                    },
                                    GameSceneDestination {
                                        scene: GameSceneId(1),
                                        position: Vec3::new(1300., 50., Layers::Player.as_f32()),
                                    },
                                )
                            };
                        println!("world: {:?}, skin: {:?}", game_scene_id, skin);

                        // Create Game Scene
                        let base = BaseScene::new();
                        let server_components = (Owner(*client_id), game_scene_id);
                        commands.spawn((base.main_building, server_components));
                        commands.spawn((base.archer_building, server_components));
                        commands.spawn((base.warrior_building, server_components));
                        commands.spawn((base.pikeman_building, server_components));
                        commands.spawn((base.left_wall, server_components));
                        commands.spawn((base.right_wall, server_components));

                        commands.spawn((
                            base.left_spawn_point,
                            server_components,
                            left_destination,
                        ));
                        commands.spawn((
                            base.right_spawn_point,
                            server_components,
                            right_destination,
                        ));

                        let game_scene_type = GameSceneType::Base(Color::from(color));
                        let game_scene = GameScene {
                            id: game_scene_id,
                            game_scene_type,
                            left_game_scenes: Vec::new(),
                            right_game_scenes: Vec::new(),
                        };
                        game_world.game_scenes.insert(game_scene_id, game_scene);

                        // Create Player entity
                        let transform = Transform::from_xyz(0., 50., Layers::Player.as_f32());
                        commands.entity(*player_entity).insert((
                            transform,
                            PlayerInput::default(),
                            Velocity::default(),
                            game_scene_id,
                            skin,
                        ));

                        let message = ServerMessages::LoadGameScene {
                            game_scene_type,
                            players: vec![SpawnPlayer {
                                id: *client_id,
                                entity: *player_entity,
                                translation: transform.translation.into(),
                                skin,
                            }],
                            units: Vec::new(),
                            projectiles: Vec::new(),
                            flag: None,
                        };
                        let message = bincode::serialize(&message).unwrap();
                        server.send_message(*client_id, ServerChannel::ServerMessages, message);
                    }

                    // setup duel map
                    let mut iter = game_world.game_scenes.iter_mut();
                    if let Some((first_game_scene_id, first_game_scene)) = iter.next() {
                        if let Some((second_game_scene_id, second_game_scene)) = iter.next() {
                            first_game_scene
                                .left_game_scenes
                                .push(*second_game_scene_id);
                            first_game_scene
                                .right_game_scenes
                                .push(*second_game_scene_id);

                            second_game_scene
                                .left_game_scenes
                                .push(*first_game_scene_id);
                            second_game_scene
                                .right_game_scenes
                                .push(*first_game_scene_id);
                        }
                    }

                    next_state.set(GameState::GameSession);
                }
                PlayerCommand::Interact => {
                    interact.send(InteractEvent(client_id));
                }

                PlayerCommand::LobbyReadyState(checkbox_state) => {
                    player_checkbox.send(PlayerChangedReady {
                        id: client_id,
                        ready_state: checkbox_state,
                    });
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

            let message = ServerMessages::DespawnEntity {
                entities: vec![entity],
            };
            let unit_dead_message = bincode::serialize(&message).unwrap();
            server.broadcast_message(ServerChannel::ServerMessages, unit_dead_message);
        }
    }
}
