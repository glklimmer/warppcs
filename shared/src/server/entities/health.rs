use bevy::prelude::*;
use bevy_renet::renet::RenetServer;

use crate::{
    map::{
        buildings::{BuildStatus, Building},
        scenes::SceneBuildingIndicator,
        GameSceneId,
    },
    networking::{BuildingUpdate, Facing, Owner, ServerChannel, ServerMessages, UpdateType},
    server::{networking::SendServerMessage, physics::movement::Velocity},
    BoxCollider, DelayedDespawn,
};

use super::Unit;

#[derive(Component, Clone)]
pub struct Health {
    pub hitpoints: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self { hitpoints: 200. }
    }
}

#[derive(Event)]
pub struct TakeDamage {
    pub target_entity: Entity,
    pub damage: f32,
    pub direction: Facing,
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamage>();

        app.add_systems(FixedUpdate, (apply_damage).run_if(on_event::<TakeDamage>));

        app.add_systems(
            FixedUpdate,
            (on_unit_death, on_building_destroy, delayed_despawn),
        );
    }
}

fn apply_damage(
    mut query: Query<(Entity, &mut Health, &GameSceneId)>,
    mut attack_events: EventReader<TakeDamage>,
    mut sender: EventWriter<SendServerMessage>,
) {
    for event in attack_events.read() {
        if let Ok((entity, mut health, game_scene_id)) = query.get_mut(event.target_entity) {
            health.hitpoints -= event.damage;
            println!("New health: {}.", health.hitpoints);

            sender.send(SendServerMessage {
                message: ServerMessages::EntityHit { entity },
                game_scene_id: *game_scene_id,
            });
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
            commands
                .entity(entity)
                .insert(DelayedDespawn(Timer::from_seconds(600., TimerMode::Once)))
                .remove::<Health>()
                .remove::<Velocity>()
                .remove::<Unit>()
                .remove::<BoxCollider>();

            sender.send(SendServerMessage {
                message: ServerMessages::EntityDeath { entity },
                game_scene_id: *game_scene_id,
            });
        }
    }
}

fn on_building_destroy(
    mut commands: Commands,
    mut sender: EventWriter<SendServerMessage>,
    mut server: ResMut<RenetServer>,
    query: Query<(
        Entity,
        &Health,
        &GameSceneId,
        &SceneBuildingIndicator,
        &Building,
        &Owner,
    )>,
) {
    for (entity, health, game_scene_id, indicator, building, owner) in query.iter() {
        if health.hitpoints <= 0. {
            commands.entity(entity).despawn_recursive();

            sender.send(SendServerMessage {
                message: ServerMessages::BuildingUpdate(BuildingUpdate {
                    indicator: *indicator,
                    update: UpdateType::Status {
                        new_status: BuildStatus::Destroyed,
                    },
                }),
                game_scene_id: *game_scene_id,
            });

            if let Building::MainBuilding { level: _ } = building {
                let message = ServerMessages::PlayerDefeat(*owner);
                let message = bincode::serialize(&message).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }
}

fn delayed_despawn(
    mut commands: Commands,
    mut query: Query<(Entity, &GameSceneId, &mut DelayedDespawn)>,
    mut sender: EventWriter<SendServerMessage>,
    time: Res<Time>,
) {
    for (entity, game_scene_id, mut delayed) in &mut query {
        let timer = &mut delayed.0;
        timer.tick(time.delta());

        if timer.just_finished() {
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
