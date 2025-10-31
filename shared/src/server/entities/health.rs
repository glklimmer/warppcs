use bevy::prelude::*;

use bevy::ecs::entity::MapEntities;
use bevy_replicon::prelude::{SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    AnimationChange, AnimationChangeEvent, DelayedDespawn, Hitby, Owner,
    map::buildings::{BuildStatus, Building, BuildingType, HealthIndicator},
    networking::{UnitType, WorldDirection},
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
            UnitType::Archer => 3,
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
                    apply_damage,
                    (on_building_destroy, on_unit_death),
                    update_build_status,
                )
                    .chain(),
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
) -> Result {
    for (entity, mut delay) in query.iter_mut() {
        delay.timer.tick(time.delta());
        if delay.timer.finished() {
            attack_events.write(delay.damage.clone());
            commands.entity(entity).despawn();
        }
    }
    Ok(())
}

fn apply_damage(
    mut attack_events: EventReader<TakeDamage>,
    mut query: Query<(Entity, &mut Health)>,
    mut animation: EventWriter<ToClients<AnimationChangeEvent>>,
) -> Result {
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
    Ok(())
}

fn update_build_status(
    mut query: Query<(&Health, &mut BuildStatus, &Building), Changed<Health>>,
) -> Result {
    for (health, mut status, building) in query.iter_mut() {
        let percentage = health.hitpoints / building.health().hitpoints * 100.0;
        let percentage_i32 = percentage.clamp(0.0, 100.0) as i32;

        let severity = match percentage_i32 {
            90..=100 => HealthIndicator::Healthy,
            70..90 => HealthIndicator::Light,
            30..70 => HealthIndicator::Medium,
            _ => HealthIndicator::Heavy,
        };

        if let BuildStatus::Built { indicator } = *status
            && indicator != severity
        {
            *status = BuildStatus::Built {
                indicator: severity,
            };
        }
    }
    Ok(())
}

fn on_unit_death(
    mut damage_events: EventReader<TakeDamage>,
    mut unit_animation: EventWriter<ToClients<AnimationChangeEvent>>,
    units: Query<
        (
            Entity,
            &Health,
            &Owner,
            Option<&TargetedBy>,
            Option<&FlagAssignment>,
            Option<&ArmyFlagAssignments>,
        ),
        With<Unit>,
    >,
    group: Query<&FlagUnits>,
    transform: Query<&Transform>,
    holder: Query<&FlagHolder>,
    mut commands: Commands,
) -> Result {
    for damage_event in damage_events.read() {
        let Ok((entity, health, owner, maybe_targeted_by, maybe_flag_assignment, maybe_army)) =
            units.get(damage_event.target_entity)
        else {
            continue;
        };
        if health.hitpoints > 0. {
            continue;
        }

        commands
            .entity(entity)
            .insert(DelayedDespawn(Timer::from_seconds(600., TimerMode::Once)))
            .despawn_related::<BehaveSources>()
            .remove::<Health>()
            .try_remove::<Interactable>();

        unit_animation.write(ToClients {
            mode: SendMode::Broadcast,
            event: AnimationChangeEvent {
                entity,
                change: AnimationChange::Death,
            },
        });

        if let Some(targeted_by) = maybe_targeted_by {
            commands
                .entity(entity)
                .remove_related::<Target>(targeted_by);
        };

        let Some(flag_assignment) = maybe_flag_assignment else {
            commands.entity(entity).remove::<BanditBehaviour>();
            continue;
        };

        commands
            .entity(entity)
            .remove::<FlagAssignment>()
            .remove::<UnitBehaviour>();

        let flag = flag_assignment.entity();
        let group = group.get(flag)?;
        let num_alive = group.len();

        // last unit from flag died
        if num_alive == 1 {
            let flag_transform = transform.get(flag)?;

            commands
                .entity(flag)
                .insert((
                    DelayedDespawn(Timer::from_seconds(620., TimerMode::Once)),
                    FlagDestroyed,
                ))
                .remove::<AttachedTo>()
                .remove::<Interactable>();

            let Ok(player) = owner.entity() else {
                continue;
            };

            if let Ok(holder) = holder.get(player)
                && flag.eq(&**holder)
            {
                commands.entity(player).remove::<FlagHolder>();
            }

            if let Some(army) = maybe_army {
                for formation_flag in army.flags.iter().flatten() {
                    commands.entity(*formation_flag).remove::<AttachedTo>();
                    commands.entity(*formation_flag).insert((
                        *flag_transform,
                        Velocity(Vec2::new((fastrand::f32() - 0.5) * 150., 100.)),
                        Visibility::Visible,
                        Interactable {
                            kind: InteractionType::Flag,
                            restricted_to: Some(player),
                        },
                    ));
                }
            }
        }
    }
    Ok(())
}

#[derive(Event, Clone, Copy, Deserialize, Serialize, Deref)]
pub struct PlayerDefeated(Entity);

impl MapEntities for PlayerDefeated {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.get_mapped(self.0);
    }
}

fn on_building_destroy(
    mut query: Query<(
        Entity,
        &Health,
        &Building,
        &mut BuildStatus,
        &Owner,
        Option<&TargetedBy>,
    )>,
    mut commands: Commands,
) -> Result {
    for (entity, health, building, mut status, owner, maybe_targeted_by) in query.iter_mut() {
        if health.hitpoints <= 0. {
            *status = BuildStatus::Destroyed;

            commands
                .entity(entity)
                .remove::<Health>()
                .insert(Interactable {
                    kind: InteractionType::Building,
                    restricted_to: Some(owner.entity()?),
                });

            if let Some(targeted_by) = maybe_targeted_by {
                commands
                    .entity(entity)
                    .remove_related::<Target>(targeted_by);
            };

            if let BuildingType::MainBuilding { level: _ } = building.building_type {
                commands.server_trigger(ToClients {
                    mode: SendMode::Broadcast,
                    event: PlayerDefeated(owner.entity()?),
                });
            }
        }
    }
    Ok(())
}

fn delayed_despawn(
    mut query: Query<(Entity, &mut DelayedDespawn)>,
    mut commands: Commands,
    time: Res<Time>,
) -> Result {
    for (entity, mut delayed) in &mut query {
        let timer = &mut delayed.0;
        timer.tick(time.delta());

        if timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
    Ok(())
}
