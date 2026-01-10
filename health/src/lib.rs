use bevy::prelude::*;

use bevy_replicon::prelude::{SendMode, ToClients};
use physics::{WorldDirection, movement::Unmovable};
use shared::{AnimationChange, AnimationChangeEvent, Hitby};

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub hitpoints: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self { hitpoints: 200. }
    }
}

#[derive(Message, Debug, Clone)]
pub struct TakeDamage {
    pub target_entity: Entity,
    pub damage: f32,
    pub direction: WorldDirection,
    pub by: Hitby,
}

#[derive(Component)]
pub struct DelayedDamage {
    pub timer: Timer,
    pub damage: TakeDamage,
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TakeDamage>().add_systems(
            FixedUpdate,
            ((delayed_damage, apply_damage).chain(), delayed_despawn),
        );
    }
}

#[derive(Component)]
pub struct DelayedDespawn(pub Timer);

fn delayed_damage(
    mut query: Query<(Entity, &mut DelayedDamage)>,
    mut attack_events: MessageWriter<TakeDamage>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut delay) in query.iter_mut() {
        delay.timer.tick(time.delta());
        if delay.timer.is_finished() {
            attack_events.write(delay.damage.clone());
            commands.entity(entity).despawn();
        }
    }
}

fn apply_damage(
    mut commands: Commands,
    mut attack_events: MessageReader<TakeDamage>,
    mut query: Query<(Entity, &mut Health)>,
    mut animation: MessageWriter<ToClients<AnimationChangeEvent>>,
) {
    for event in attack_events.read() {
        let Ok((entity, mut health)) = query.get_mut(event.target_entity) else {
            continue;
        };

        health.hitpoints -= event.damage;

        if health.hitpoints <= 0. {
            commands.entity(entity).remove::<Health>().insert(Unmovable);
        }

        animation.write(ToClients {
            mode: SendMode::Broadcast,
            message: AnimationChangeEvent {
                entity,
                change: AnimationChange::Hit(event.by),
            },
        });
    }
}

fn delayed_despawn(
    mut query: Query<(Entity, &mut DelayedDespawn)>,
    mut commands: Commands,
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
