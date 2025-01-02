use bevy::prelude::*;
use start_game::StartGamePlugin;

use crate::{
    map::{buildings::BuildStatus, scenes::SceneBuildingIndicator, GameSceneId},
    networking::{
        BuildingUpdate, MultiplayerRoles, Owner, PlayerSkin, ProjectileType, ServerChannel,
        ServerMessages, SpawnFlag, SpawnPlayer, SpawnProjectile, SpawnUnit,
    },
    BoxCollider, GameState,
};
use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy_renet::renet::RenetServer;

use super::{
    buildings::recruiting::{FlagAssignment, FlagHolder},
    entities::Unit,
    networking::{GameWorld, SendServerMessage, ServerLobby, ServerPlayer},
    physics::movement::Velocity,
    players::InteractEvent,
};

pub mod start_game;

#[derive(Component, Clone)]
pub struct GameSceneDestination {
    pub scene: GameSceneId,
    pub position: Vec3,
}

#[derive(Event)]
pub struct TravelEvent {
    pub entity: Entity,
    pub target: GameSceneDestination,
}

pub struct GameScenesPlugin;

impl Plugin for GameScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StartGamePlugin);

        app.add_event::<TravelEvent>();

        app.add_systems(
            FixedUpdate,
            (check_travel, travel)
                .run_if(in_state(GameState::GameSession).and(in_state(MultiplayerRoles::Host))),
        );
    }
}

fn check_travel(
    lobby: Res<ServerLobby>,
    player: Query<(&Transform, &BoxCollider, &GameSceneId)>,
    zones: Query<(
        &Transform,
        &BoxCollider,
        &GameSceneDestination,
        &GameSceneId,
    )>,
    mut travel: EventWriter<TravelEvent>,
    mut interactions: EventReader<InteractEvent>,
) {
    for event in interactions.read() {
        let client_id = event.0;
        let player_entity = lobby.players.get(&client_id).unwrap();

        let (player_transform, player_collider, player_scene) = player.get(*player_entity).unwrap();
        for (zone_transform, zone_collider, destination, zone_scene) in zones.iter() {
            if player_scene.ne(zone_scene) {
                continue;
            }
            let player_bounds = Aabb2d::new(
                player_transform.translation.truncate(),
                player_collider.half_size(),
            );
            let zone_bounds = Aabb2d::new(
                zone_transform.translation.truncate(),
                zone_collider.half_size(),
            );
            if player_bounds.intersects(&zone_bounds) {
                travel.send(TravelEvent {
                    entity: *player_entity,
                    target: destination.clone(),
                });
            }
        }
    }
}

struct FlagGroup<'a> {
    flag: Entity,
    units: Vec<(Entity, &'a FlagAssignment, &'a Unit)>,
}

#[allow(clippy::too_many_arguments)]
fn travel(
    mut commands: Commands,
    mut traveling: EventReader<TravelEvent>,
    mut server: ResMut<RenetServer>,
    lobby: Res<ServerLobby>,
    game_world: Res<GameWorld>,
    player_skins: Query<&PlayerSkin>,
    scene_ids: Query<&GameSceneId>,
    units: Query<(&GameSceneId, &Owner, Entity, &Unit, &Transform)>,
    projectiles: Query<(&GameSceneId, Entity, &ProjectileType, &Transform, &Velocity)>,
    buildings: Query<(&GameSceneId, &SceneBuildingIndicator, &BuildStatus)>,
    transforms: Query<&Transform>,
    server_players: Query<&ServerPlayer>,
    flag_holders: Query<&FlagHolder>,
    units_on_flag: Query<(Entity, &FlagAssignment, &Unit)>,
) {
    for event in traveling.read() {
        let player_entity = event.entity;
        let target_game_scene_id = event.target.scene;
        let target_position = event.target.position;
        let client_id = server_players.get(player_entity).unwrap().0;
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
        let target_transform = Transform::from_translation(target_position);
        commands
            .entity(player_entity)
            .insert((target_game_scene_id, target_transform));
        if let Some(group) = &group {
            commands
                .entity(group.flag)
                .insert((target_game_scene_id, target_transform));
            for (unit, _, _) in &group.units {
                commands
                    .entity(*unit)
                    .insert((target_game_scene_id, target_transform));
            }
        }

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
            entity: player_entity,
            translation: target_transform.translation.into(),
            skin: *player_skins.get(player_entity).unwrap(),
        });
        let flag = group.as_ref().map(|g| SpawnFlag { entity: g.flag });
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
                    owner: Owner(client_id),
                    entity: *unit,
                    translation: target_transform.translation.into(),
                    unit_type: info.unit_type,
                });
            }
        }
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
            .map(|(_, indicator, status)| BuildingUpdate {
                indicator: *indicator,
                status: *status,
            })
            .collect();

        let message = ServerMessages::LoadGameScene {
            game_scene_type: game_scene.game_scene_type,
            players,
            flag,
            units,
            projectiles,
            buildings,
        };
        let message = bincode::serialize(&message).unwrap();
        server.send_message(client_id, ServerChannel::ServerMessages, message);

        // Tell other players in new scene that new player and units arrived
        let skin = player_skins.get(player_entity).unwrap();
        let mut unit_spawns = Vec::new();
        if let Some(group) = &group {
            for (unit, _, info) in &group.units {
                unit_spawns.push(SpawnUnit {
                    owner: Owner(client_id),
                    entity: *unit,
                    translation: target_transform.translation.into(),
                    unit_type: info.unit_type,
                });
            }
        }

        let message = ServerMessages::SpawnGroup {
            player: SpawnPlayer {
                id: client_id,
                entity: player_entity,
                translation: target_transform.translation.into(),
                skin: *skin,
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
