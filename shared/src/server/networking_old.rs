use bevy::prelude::*;

use bevy::color::palettes::css::{BLUE, RED};
use renet_steam::bevy::SteamServerPlugin;

use crate::map::base::MainBuildingBundle;
use crate::map::{GameScene, GameSceneId, GameSceneType, Layers};
use crate::networking::{
    ClientChannel, Facing, NetworkEntity, NetworkedEntities, Owner, PlayerCommand, PlayerInput,
    PlayerSkin, ProjectileType, Rotation, ServerChannel, ServerMessages, SpawnPlayer,
    SpawnProjectile, SpawnUnit, Unit,
};
use crate::server::ai::attack::{unit_health, unit_swing_timer};
use crate::server::ai::UnitBehaviour;
use crate::BoxCollider;
use bevy_renet::{
    renet::{ClientId, RenetServer, ServerEvent},
    RenetServerPlugin,
};
use std::collections::HashMap;

use super::ai::AIPlugin;
use super::physics::movement::Velocity;
use super::physics::PhysicsPlugin;

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<ClientId, Entity>,
}

#[derive(Default, Resource)]
pub struct GameWorld {
    pub game_scenes: HashMap<GameSceneId, GameScene>,
}

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (server_update_system, server_network_sync, on_unit_death),
        );

        app.add_plugins(AIPlugin);
        app.add_plugins(PhysicsPlugin);

        app.insert_resource(ServerLobby::default());
        app.add_plugins(RenetServerPlugin);

        app.insert_resource(GameWorld::default());

        app.add_plugins(SteamServerPlugin);
    }
}

#[allow(clippy::too_many_arguments)]
fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    transforms: Query<&Transform>,
    player_skins: Query<&PlayerSkin>,
    scene_ids: Query<&GameSceneId>,
    units: Query<(&GameSceneId, &Owner, Entity, &Unit, &Transform)>,
    projectiles: Query<(&GameSceneId, Entity, &ProjectileType, &Transform, &Velocity)>,
    mut game_world: ResMut<GameWorld>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {} connected.", client_id);
                let player_entity = commands.spawn(()).id();
                lobby.players.insert(*client_id, player_entity);
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

                    let player_entity = lobby.players.get(&client_id).unwrap();
                    let player_transform = transforms.get(*player_entity).unwrap();
                    let scene_id = scene_ids.get(*player_entity).unwrap();
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
                            *scene_id,
                        ))
                        .id();

                    let message = ServerMessages::SpawnUnit(SpawnUnit {
                        entity: unit_entity,
                        owner: Owner(client_id),
                        unit_type,
                        translation: player_transform.translation.into(),
                    });
                    let message = bincode::serialize(&message).unwrap();
                    for (client_id, entity) in lobby.players.iter() {
                        let player_scene_id = scene_ids.get(*entity).unwrap();
                        if scene_id.eq(player_scene_id) {
                            server.send_message(
                                *client_id,
                                ServerChannel::ServerMessages,
                                message.clone(),
                            );
                        }
                    }
                }
                PlayerCommand::StartGame => {
                    println!("Starting game...");
                    for (client_id, player_entity) in lobby.players.iter() {
                        let (game_scene_id, skin, color) = if game_world.game_scenes.is_empty() {
                            (GameSceneId(1), PlayerSkin::Warrior, BLUE)
                        } else {
                            (GameSceneId(2), PlayerSkin::Monster, RED)
                        };
                        println!("world: {:?}, skin: {:?}", game_scene_id, skin);

                        // Create Game Scene
                        commands.spawn((
                            MainBuildingBundle::new(),
                            Owner(*client_id),
                            game_scene_id,
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
                }
                PlayerCommand::TravelTo(target_game_scene_id) => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        println!(
                            "player {:?} traveling to {:?}",
                            client_id, target_game_scene_id
                        );

                        // Remove entity from game scene
                        commands.entity(*player_entity).remove::<GameSceneId>();

                        // Tell all players on map that player left
                        let message = ServerMessages::DespawnEntity {
                            entity: *player_entity,
                        };
                        let message = bincode::serialize(&message).unwrap();
                        let current_game_scene_id = scene_ids.get(*player_entity).unwrap();
                        for (other_client_id, other_entity) in lobby.players.iter() {
                            let other_scene_id = scene_ids.get(*other_entity).unwrap();
                            if current_game_scene_id.eq(other_scene_id) {
                                server.send_message(
                                    *other_client_id,
                                    ServerChannel::ServerMessages,
                                    message.clone(),
                                );
                            }
                        }

                        // Travel Player to new game scene
                        let player_target_transform =
                            Transform::from_xyz(0., 50., Layers::Player.as_f32());
                        commands
                            .entity(*player_entity)
                            .insert((target_game_scene_id, player_target_transform));

                        // Tell client to load game scene
                        let game_scene = game_world.game_scenes.get(&target_game_scene_id).unwrap();
                        let mut players: Vec<SpawnPlayer> = lobby
                            .players
                            .iter()
                            .filter(|(_, other_entity)| {
                                let scene = scene_ids.get(**other_entity).unwrap();
                                target_game_scene_id.eq(scene)
                            })
                            .map(|(other_client_id, other_entity)| {
                                let transform = transforms.get(*other_entity).unwrap();
                                let skin = player_skins.get(*other_entity).unwrap();
                                SpawnPlayer {
                                    id: *other_client_id,
                                    entity: *other_entity,
                                    translation: transform.translation.into(),
                                    skin: *skin,
                                }
                            })
                            .collect();
                        players.push(SpawnPlayer {
                            id: client_id,
                            entity: *player_entity,
                            translation: player_target_transform.translation.into(),
                            skin: *player_skins.get(*player_entity).unwrap(),
                        });
                        let units = units
                            .iter()
                            .filter(|(scene, ..)| target_game_scene_id.eq(*scene))
                            .map(|(_, owner, entity, unit, translation)| SpawnUnit {
                                owner: *owner,
                                entity,
                                unit_type: unit.unit_type.clone(),
                                translation: translation.translation.into(),
                            })
                            .collect();
                        let projectiles = projectiles
                            .iter()
                            .filter(|(scene, ..)| target_game_scene_id.eq(*scene))
                            .map(
                                |(_, entity, projectile, translation, velocity)| SpawnProjectile {
                                    entity,
                                    projectile_type: *projectile,
                                    translation: translation.translation.into(),
                                    direction: velocity.0.into(),
                                },
                            )
                            .collect();

                        let message = ServerMessages::LoadGameScene {
                            game_scene_type: game_scene.game_scene_type,
                            players,
                            units,
                            projectiles,
                        };
                        let message = bincode::serialize(&message).unwrap();
                        server.send_message(client_id, ServerChannel::ServerMessages, message);

                        // Tell other players in new scene that new player arrived
                        let skin = player_skins.get(*player_entity).unwrap();
                        let message = ServerMessages::SpawnPlayer(SpawnPlayer {
                            id: client_id,
                            entity: *player_entity,
                            translation: player_target_transform.translation.into(),
                            skin: *skin,
                        });
                        let message = bincode::serialize(&message).unwrap();
                        for (other_client_id, other_entity) in lobby.players.iter() {
                            let other_scene_id = scene_ids.get(*other_entity).unwrap();
                            if target_game_scene_id.eq(other_scene_id) {
                                server.send_message(
                                    *other_client_id,
                                    ServerChannel::ServerMessages,
                                    message.clone(),
                                );
                            }
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
