use bevy::prelude::*;

use bevy_renet::renet::RenetServer;
use start_game::StartGamePlugin;

use crate::{
    map::{
        buildings::{BuildStatus, Building},
        scenes::SceneBuildingIndicator,
        GameSceneId,
    },
    networking::{
        Faction, LoadBuilding, Owner, ProjectileType, ServerChannel, ServerMessages, SpawnFlag,
        SpawnMount, SpawnPlayer, SpawnProjectile, SpawnUnit,
    },
    server::players::interaction::InteractionType,
};

use super::{
    buildings::recruiting::{FlagAssignment, FlagHolder},
    entities::{Mount, Unit},
    networking::{GameWorld, ServerLobby},
    physics::movement::Velocity,
    players::interaction::InteractionTriggeredEvent,
};

pub mod start_game;

#[derive(Component, Clone)]
pub struct GameSceneDestination {
    pub scene: GameSceneId,
    pub position: Vec3,
}

pub struct GameScenesPlugin;

impl Plugin for GameScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StartGamePlugin);

        app.add_systems(FixedUpdate, travel);
    }
}

struct FlagGroup<'a> {
    flag: Entity,
    units: Vec<(Entity, &'a FlagAssignment, &'a Unit)>,
}

#[allow(clippy::too_many_arguments)]
fn travel(
    mut commands: Commands,
    mut traveling: EventReader<InteractionTriggeredEvent>,
    mut server: ResMut<RenetServer>,
    lobby: Res<ServerLobby>,
    game_world: Res<GameWorld>,
    scene_ids: Query<&GameSceneId>,
    units: Query<(&GameSceneId, &Owner, Entity, &Unit, &Transform)>,
    mounts: Query<(&GameSceneId, Entity, &Mount, &Transform)>,
    projectiles: Query<(&GameSceneId, Entity, &ProjectileType, &Transform, &Velocity)>,
    buildings: Query<(
        &GameSceneId,
        &SceneBuildingIndicator,
        &BuildStatus,
        &Building,
    )>,
    transforms: Query<&Transform>,
    flag_holders: Query<&FlagHolder>,
    units_on_flag: Query<(Entity, &FlagAssignment, &Unit)>,
    destination: Query<&GameSceneDestination>,
) {
    for event in traveling.read() {
        let InteractionType::Travel = &event.interaction else {
            continue;
        };

        let player_entity = event.player;
        let GameSceneDestination {
            scene: target_game_scene_id,
            position: target_position,
        } = destination.get(event.interactable).unwrap();
        println!(
            "travel happening... destination: {:?}",
            target_game_scene_id
        );
        let group = match flag_holders.get(player_entity) {
            Ok(flag_holder) => Some(FlagGroup {
                flag: flag_holder.0,
                units: units_on_flag
                    .iter()
                    .filter(|(_, assignment, _)| assignment.0.eq(&flag_holder.0))
                    .collect(),
            }),
            Err(_) => None,
        };

        println!("target: {:?}", target_game_scene_id);

        // Remove player, flag and units from game scene
        commands.entity(player_entity).remove::<GameSceneId>();
        if let Some(group) = &group {
            commands.entity(group.flag).remove::<GameSceneId>();

            for (unit, _, _) in &group.units {
                commands.entity(*unit).remove::<GameSceneId>();
            }
        }

        // Tell all players on map that player and units left
        let mut entities = Vec::new();
        entities.push(player_entity);
        if let Some(group) = &group {
            let mut units: Vec<Entity> = group.units.iter().map(|unit| unit.0).collect();
            entities.append(&mut units);
        }
        let current_game_scene_id = scene_ids.get(player_entity).unwrap();

        let message = ServerMessages::DespawnEntity { entities };
        let message = bincode::serialize(&message).unwrap();
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

        // Travel Player, flag and units to new game scene
        let target_transform = Transform::from_translation(*target_position);
        commands
            .entity(player_entity)
            .insert((*target_game_scene_id, target_transform));
        if let Some(group) = &group {
            commands
                .entity(group.flag)
                .insert((*target_game_scene_id, target_transform));
            for (unit, _, _) in &group.units {
                commands
                    .entity(*unit)
                    .insert((*target_game_scene_id, target_transform));
            }
        }

        // Tell client to load game scene
        let game_scene = game_world.game_scenes.get(target_game_scene_id).unwrap();
        let mut players: Vec<SpawnPlayer> = lobby
            .players
            .iter()
            .filter(|(_, other_entity)| {
                let scene = scene_ids.get(**other_entity).unwrap();
                target_game_scene_id.eq(scene)
            })
            .map(|(other_client_id, other_entity)| {
                let transform = transforms.get(*other_entity).unwrap();
                SpawnPlayer {
                    id: *other_client_id,
                    entity: *other_entity,
                    translation: transform.translation.into(),
                }
            })
            .collect();
        players.push(SpawnPlayer {
            id: event.client_id,
            entity: player_entity,
            translation: target_transform.translation.into(),
        });
        let flag = group.as_ref().map(|g| SpawnFlag { flag: g.flag });
        let mut units: Vec<SpawnUnit> = units
            .iter()
            .filter(|(scene, ..)| target_game_scene_id.eq(*scene))
            .map(|(_, owner, entity, unit, translation)| SpawnUnit {
                owner: *owner,
                entity,
                unit_type: unit.unit_type,
                translation: translation.translation.into(),
            })
            .collect();
        if let Some(group) = &group {
            for (unit, _, info) in &group.units {
                units.push(SpawnUnit {
                    owner: Owner {
                        faction: Faction::Player {
                            client_id: event.client_id,
                        },
                    },
                    entity: *unit,
                    translation: target_transform.translation.into(),
                    unit_type: info.unit_type,
                });
            }
        }
        let mounts: Vec<SpawnMount> = mounts
            .iter()
            .filter(|(scene, ..)| target_game_scene_id.eq(*scene))
            .map(|(_, entity, mount, translation)| SpawnMount {
                entity,
                mount_type: mount.mount_type,
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

        let buildings = buildings
            .iter()
            .filter(|(scene, ..)| target_game_scene_id.eq(*scene))
            .map(|(_, indicator, status, building)| LoadBuilding {
                indicator: *indicator,
                status: *status,
                upgrade: *building,
            })
            .collect();

        let message = ServerMessages::LoadGameScene {
            game_scene_type: game_scene.game_scene_type,
            players,
            flag,
            units,
            mounts,
            projectiles,
            buildings,
        };
        let message = bincode::serialize(&message).unwrap();
        server.send_message(event.client_id, ServerChannel::ServerMessages, message);

        // Tell other players in new scene that new player and units arrived
        let mut unit_spawns = Vec::new();
        if let Some(group) = &group {
            for (unit, _, info) in &group.units {
                unit_spawns.push(SpawnUnit {
                    owner: Owner {
                        faction: Faction::Player {
                            client_id: event.client_id,
                        },
                    },
                    entity: *unit,
                    translation: target_transform.translation.into(),
                    unit_type: info.unit_type,
                });
            }
        }

        let message = ServerMessages::SpawnGroup {
            player: SpawnPlayer {
                id: event.client_id,
                entity: player_entity,
                translation: target_transform.translation.into(),
            },
            units: unit_spawns,
        };
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
