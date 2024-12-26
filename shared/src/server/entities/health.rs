use bevy::prelude::*;

use crate::{
    map::{buildings::BuildStatus, scenes::SceneBuildingIndicator, GameSceneId},
    networking::{BuildingUpdate, MultiplayerRoles, ServerMessages},
    server::networking::SendServerMessage,
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
    mut commands: Commands,
    mut sender: EventWriter<SendServerMessage>,
    query: Query<(Entity, &Health, &GameSceneId), With<Unit>>,
) {
    for (entity, health, game_scene_id) in query.iter() {
        if health.hitpoints <= 0. {
            commands.entity(entity).despawn_recursive();

            sender.send(SendServerMessage {
                message: ServerMessages::DespawnEntity {
                    entities: vec![entity],
                },
                game_scene_id: *game_scene_id,
            });
        }
    }
}

fn on_building_destroy(
    mut commands: Commands,
    mut sender: EventWriter<SendServerMessage>,
    query: Query<(Entity, &Health, &GameSceneId, &SceneBuildingIndicator)>,
) {
    for (entity, health, game_scene_id, indicator) in query.iter() {
        if health.hitpoints <= 0. {
            commands.entity(entity).despawn_recursive();

            sender.send(SendServerMessage {
                message: ServerMessages::BuildingUpdate(BuildingUpdate {
                    indicator: *indicator,
                    status: BuildStatus::Destroyed,
                }),
                game_scene_id: *game_scene_id,
            });
        }
    }
}
