use bevy::prelude::*;

use bevy_replicon::prelude::{SendMode, ToClients};

use crate::{map::buildings::{Building, BuildingType}, networking::{UnitType, WorldDirection}, server::{ai::{Target, TargetedBy}, buildings::recruiting::FlagAssignment}, AnimationChange, AnimationChangeEvent, DelayedDespawn, Hitby, Owner};

use super::Unit;

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub hitpoints: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self { hitpoints: 200. }
    }
}

#[derive(Event, Debug, Clone)]
pub struct TakeDamage {
    pub target_entity: Entity,
    pub damage: f32,
    pub direction: WorldDirection,
    pub by: Hitby,
}

#[derive(Component)]
pub struct DelayedDamage {
    timer: Timer,
    damage: TakeDamage,
}

impl DelayedDamage {
    pub fn new(unit_type: &UnitType, damage: TakeDamage) -> Self {
        let frame_delay = match unit_type {
            UnitType::Shieldwarrior => 2,
            UnitType::Pikeman => 3,
            UnitType::Archer => todo!(),
            UnitType::Bandit => 2,
            UnitType::Commander => 2,
        };

        let duration = frame_delay as f32 * 0.1;

        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            damage,
        }
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamage>();

        app.add_systems(
            FixedUpdate,
            (
                (
                    delayed_damage,
                    (apply_damage).run_if(on_event::<TakeDamage>),
                )
                    .chain(),
                on_unit_death,
                on_building_destroy,
                delayed_despawn,
            ),
        );
    }
}

fn delayed_damage(
    mut query: Query<(Entity, &mut DelayedDamage)>,
    mut attack_events: EventWriter<TakeDamage>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut delay) in query.iter_mut() {
        delay.timer.tick(time.delta());
        if delay.timer.finished() {
            attack_events.write(delay.damage.clone());
            commands.entity(entity).despawn();
        }
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

            animation.write(ToClients {
                mode: SendMode::Broadcast,
                event: AnimationChangeEvent {
                    entity,
                    change: AnimationChange::Hit(event.by),
                },
            });
        }
    }
}

fn on_unit_death(
    mut commands: Commands,
    mut animation: EventWriter<ToClients<AnimationChangeEvent>>,
    query: Query<(Entity, &Health, &TargetedBy), With<Unit>>,
) {
    for (entity, health, targeted_by) in query.iter() {
        if health.hitpoints <= 0. {
            commands
                .entity(entity)
                .insert(DelayedDespawn(Timer::from_seconds(600., TimerMode::Once)))
                .remove::<FlagAssignment>()
                .remove_related::<Target>(targeted_by)
                .remove::<Health>();

            animation.write(ToClients {
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
            commands.entity(entity).despawn();

            if let BuildingType::MainBuilding { level: _ } = building.building_type {
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
            commands.entity(entity).despawn();
        }
    }
}
