use bevy::prelude::*;

use bevy_replicon::prelude::{SendMode, ToClients};

use crate::{
    AnimationChange, AnimationChangeEvent, BoxCollider, DelayedDespawn, FlagAnimation,
    FlagAnimationEvent, Hitby, Owner,
    map::buildings::{BuildStatus, Building, BuildingType, HealthIndicator},
    networking::{UnitType, WorldDirection},
    server::{
        ai::{BehaveSources, Target, TargetedBy, UnitBehaviour},
        buildings::recruiting::{FlagAssignment, FlagHolder, FlagUnits},
        physics::{attachment::AttachedTo, movement::Velocity},
        players::interaction::{Interactable, InteractionType},
    },
};

use super::{Unit, commander::ArmyFlagAssignments};

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
                    update_build_status,
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

fn update_build_status(mut query: Query<(&Health, &mut BuildStatus, &Building), Changed<Health>>) {
    for (health, mut status, building) in query.iter_mut() {
        let percentage = health.hitpoints / building.health().hitpoints * 100.0;
        let percentage_i32 = percentage.clamp(0.0, 100.0) as i32;

        let severity = match percentage_i32 {
            90..=100 => HealthIndicator::Healthy,
            70..90 => HealthIndicator::Light,
            30..70 => HealthIndicator::Medium,
            _ => HealthIndicator::Heavy,
        };

        if let BuildStatus::Built { indicator } = *status {
            if indicator != severity {
                *status = BuildStatus::Built {
                    indicator: severity,
                };
            }
        }
    }
}

fn on_unit_death(
    mut commands: Commands,
    mut unit_animation: EventWriter<ToClients<AnimationChangeEvent>>,
    mut flag_animation: EventWriter<ToClients<FlagAnimationEvent>>,
    units: Query<
        (
            Entity,
            &Health,
            &TargetedBy,
            &FlagAssignment,
            &Owner,
            Option<&ArmyFlagAssignments>,
        ),
        With<Unit>,
    >,
    group: Query<&FlagUnits>,
    transform: Query<&Transform>,
    holder: Query<&FlagHolder>,
) {
    for (entity, health, targeted_by, flag_assignment, owner, maybe_army) in units.iter() {
        if health.hitpoints > 0. {
            continue;
        }

        commands
            .entity(entity)
            .insert(DelayedDespawn(Timer::from_seconds(600., TimerMode::Once)))
            .remove::<FlagAssignment>()
            .despawn_related::<BehaveSources>()
            .remove::<UnitBehaviour>()
            .remove_related::<Target>(targeted_by)
            .remove::<Interactable>()
            .remove::<Health>();

        unit_animation.write(ToClients {
            mode: SendMode::Broadcast,
            event: AnimationChangeEvent {
                entity,
                change: AnimationChange::Death,
            },
        });

        let flag = flag_assignment.entity();
        let group = group.get(flag).unwrap();
        let num_alive = group.len();

        // last unit from flag died
        if num_alive == 1 {
            let flag_transform = transform.get(flag).unwrap();

            commands
                .entity(flag)
                .insert(DelayedDespawn(Timer::from_seconds(620., TimerMode::Once)))
                .remove::<AttachedTo>()
                .remove::<Interactable>();

            if let Some(player) = owner.entity() {
                if let Ok(holder) = holder.get(player) {
                    if flag.eq(&**holder) {
                        commands.entity(player).remove::<FlagHolder>();
                    }
                }
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
                            restricted_to: owner.entity(),
                        },
                    ));
                }
            }

            flag_animation.write(ToClients {
                mode: SendMode::Broadcast,
                event: FlagAnimationEvent {
                    entity: flag,
                    animation: FlagAnimation::Destroyed,
                },
            });
        }
    }
}

fn on_building_destroy(
    mut commands: Commands,
    mut query: Query<(Entity, &Health, &Building, &mut BuildStatus, &TargetedBy)>,
) {
    for (entity, health, building, mut status, targeted_by) in query.iter_mut() {
        if health.hitpoints <= 0. {
            *status = BuildStatus::Destroyed;

            commands
                .entity(entity)
                .remove::<Health>()
                .remove::<BoxCollider>()
                .remove_related::<Target>(targeted_by);

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
