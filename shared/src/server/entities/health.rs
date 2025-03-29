use bevy::prelude::*;

use bevy_replicon::prelude::{SendMode, ToClients};

use crate::{
    map::buildings::Building, networking::Facing, server::physics::movement::Velocity,
    AnimationChange, AnimationChangeEvent, BoxCollider, DelayedDespawn, Owner,
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
    mut query: Query<(Entity, &mut Health)>,
    mut attack_events: EventReader<TakeDamage>,
    mut animation: EventWriter<ToClients<AnimationChangeEvent>>,
) {
    for event in attack_events.read() {
        if let Ok((entity, mut health)) = query.get_mut(event.target_entity) {
            health.hitpoints -= event.damage;
            println!("New health: {}.", health.hitpoints);

            animation.send(ToClients {
                mode: SendMode::Broadcast,
                event: AnimationChangeEvent {
                    entity,
                    change: AnimationChange::Hit,
                },
            });
        }
    }
}

fn on_unit_death(
    mut commands: Commands,
    mut animation: EventWriter<ToClients<AnimationChangeEvent>>,
    query: Query<(Entity, &Health), With<Unit>>,
) {
    for (entity, health) in query.iter() {
        if health.hitpoints <= 0. {
            commands
                .entity(entity)
                .insert(DelayedDespawn(Timer::from_seconds(600., TimerMode::Once)))
                .remove::<Health>()
                .remove::<Velocity>()
                .remove::<Unit>()
                .remove::<BoxCollider>();

            animation.send(ToClients {
                mode: SendMode::Broadcast,
                event: AnimationChangeEvent {
                    entity,
                    change: AnimationChange::Death,
                },
            });
        }
    }
}

fn on_building_destroy(mut commands: Commands, query: Query<(Entity, &Health, &Building, &Owner)>) {
    for (entity, health, building, _) in query.iter() {
        if health.hitpoints <= 0. {
            commands.entity(entity).despawn_recursive();

            if let Building::MainBuilding { level: _ } = building {
                // TODO: handle player dead
            }
        }
    }
}

fn delayed_despawn(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DelayedDespawn)>,
    time: Res<Time>,
) {
    for (entity, mut delayed) in &mut query {
        let timer = &mut delayed.0;
        timer.tick(time.delta());

        if timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
