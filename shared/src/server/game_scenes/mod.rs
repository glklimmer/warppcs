use bevy::prelude::*;

use crate::map::GameSceneId;
use crate::networking::{
    Owner, PlayerSkin, ProjectileType, ServerChannel, ServerMessages, SpawnPlayer, SpawnProjectile,
    SpawnUnit, Unit,
};
use crate::{BoxCollider, GameState};
use bevy::math::bounding::Aabb2d;
use bevy::math::bounding::IntersectsVolume;
use bevy_renet::renet::RenetServer;

use super::networking::{GameWorld, InteractEvent, ServerLobby, ServerPlayer};
use super::physics::movement::Velocity;

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
        app.add_event::<TravelEvent>();

        app.add_systems(
            FixedUpdate,
            (check_travel, travel).run_if(in_state(GameState::GameSession)),
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
    transforms: Query<&Transform>,
    server_players: Query<&ServerPlayer>,
) {
    for event in traveling.read() {
        let player_entity = event.entity;
        let target_game_scene_id = event.target.scene;
        let target_position = event.target.position;
        let client_id = server_players.get(player_entity).unwrap().0;

        println!("target: {:?}", target_game_scene_id);

        // Remove entity from game scene
        commands.entity(player_entity).remove::<GameSceneId>();

        // Tell all players on map that player left
        let message = ServerMessages::DespawnEntity {
            entity: player_entity,
        };
        let message = bincode::serialize(&message).unwrap();
        let current_game_scene_id = scene_ids.get(player_entity).unwrap();
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
        let player_target_transform = Transform::from_translation(target_position);
        commands
            .entity(player_entity)
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
            entity: player_entity,
            translation: player_target_transform.translation.into(),
            skin: *player_skins.get(player_entity).unwrap(),
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
        let skin = player_skins.get(player_entity).unwrap();
        let message = ServerMessages::SpawnPlayer(SpawnPlayer {
            id: client_id,
            entity: player_entity,
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
