use bevy::prelude::*;

use bevy::ecs::entity::MapEntities;
use bevy_replicon::prelude::{SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use shared::{
    AnimationChange, AnimationChangeEvent, DelayedDespawn, Hitby, Owner,
    networking::WorldDirection,
    server::{
        ai::{BanditBehaviour, BehaveSources, Target, TargetedBy, UnitBehaviour},
        buildings::recruiting::{FlagAssignment, FlagHolder, FlagUnits},
        entities::Unit,
        physics::{attachment::AttachedTo, movement::Velocity},
        players::{
            flag::FlagDestroyed,
            interaction::{Interactable, InteractionType},
        },
    },
};

use super::commander::ArmyFlagAssignments;

// TODO: We need an event to register when something (unit/building/etc) dies.

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
        app.add_message::<TakeDamage>();

        app.add_systems(
            FixedUpdate,
            ((delayed_damage, apply_damage).chain(), delayed_despawn),
        );
    }
}

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
    mut attack_events: MessageReader<TakeDamage>,
    mut query: Query<(Entity, &mut Health)>,
    mut animation: MessageWriter<ToClients<AnimationChangeEvent>>,
) {
    for event in attack_events.read() {
        if let Ok((entity, mut health)) = query.get_mut(event.target_entity) {
            health.hitpoints -= event.damage;

            animation.write(ToClients {
                mode: SendMode::Broadcast,
                message: AnimationChangeEvent {
                    entity,
                    change: AnimationChange::Hit(event.by),
                },
            });
        }
    }
}

#[derive(Event, Clone, Copy, Deserialize, Serialize, Deref)]
pub struct PlayerDefeated(Entity);

impl MapEntities for PlayerDefeated {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.get_mapped(self.0);
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
