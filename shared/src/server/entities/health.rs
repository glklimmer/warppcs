use bevy::prelude::*;

use bevy_renet::renet::RenetServer;

use crate::{
    map::{
        buildings::{BuildStatus, Building},
        scenes::SceneBuildingIndicator,
        GameSceneId,
    },
    networking::{BuildingUpdate, GameSceneAware, MultiplayerRoles, ServerChannel, ServerMessages},
    server::networking::ServerLobby,
    GameState,
};

use super::Unit;

#[derive(Component, Clone)]
pub struct Health {
    pub hitpoints: f32,
}

#[derive(Event)]
pub struct TakeDamage {
    pub target_entity: Entity,
    pub damage: f32,
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamage>();

        app.add_systems(FixedUpdate, (apply_damage).run_if(on_event::<TakeDamage>()));

        app.add_systems(
            FixedUpdate,
            (on_unit_death, on_building_destroy).run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
    }
}

fn apply_damage(mut query: Query<&mut Health>, mut attack_events: EventReader<TakeDamage>) {
    for event in attack_events.read() {
        if let Ok(mut health) = query.get_mut(event.target_entity) {
            health.hitpoints -= event.damage;
            println!("New health: {}.", health.hitpoints);
        }
    }
}

fn on_unit_death(
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
    query: Query<(Entity, &Health, &GameSceneId), With<Unit>>,
    lobby: Res<ServerLobby>,
    scene_ids: Query<&GameSceneId>,
) {
    for (entity, health, game_scene_id) in query.iter() {
        if health.hitpoints <= 0. {
            commands.entity(entity).despawn_recursive();

            ServerMessages::DespawnEntity {
                entities: vec![entity],
            }
            .send_to_all_in_game_scene(&mut server, &lobby, &scene_ids, game_scene_id);
        }
    }
}

fn on_building_destroy(
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
    query: Query<(Entity, &Health, &GameSceneId, &SceneBuildingIndicator)>,
    lobby: Res<ServerLobby>,
    scene_ids: Query<&GameSceneId>,
) {
    for (entity, health, game_scene_id, indicator) in query.iter() {
        if health.hitpoints <= 0. {
            commands.entity(entity).despawn_recursive();

            ServerMessages::BuildingUpdate(BuildingUpdate {
                indicator: *indicator,
                status: BuildStatus::Destroyed,
            })
            .send_to_all_in_game_scene(&mut server, &lobby, &scene_ids, game_scene_id);
        }
    }
}
