use bevy::{math::ops::abs, prelude::*};

use army::flag::FlagAssignment;
use bevy::math::bounding::IntersectsVolume;
use bevy_behave::prelude::{BehaveCtx, BehaveTrigger};
use physics::{
    attachment::AttachedTo,
    movement::{BoxCollider, RandomVelocityMul, Speed, Velocity},
};
use units::{MeleeRange, ProjectileRange, Unit, UnitType};

use crate::{
    ArmyFormationTo, ArmyFormations, Reposition, RepositionTo, WalkIntoRange, WalkingInDirection,
};

use super::{FormationHasTarget, Target, offset::FollowOffset};

#[derive(Component, Clone)]
pub struct FollowFlag;

#[derive(Component, Clone)]
pub struct IsFriendlyFormationUnitInFront;

#[derive(Component, Clone)]
pub struct IsFriendlyUnitInFront;

#[derive(Component, Clone, Deref, DerefMut)]
pub struct Roam(Timer);

impl Default for Roam {
    fn default() -> Self {
        Self(Timer::from_seconds(
            fastrand::f32() * 2. + 1.,
            TimerMode::Repeating,
        ))
    }
}

pub struct AIMovementPlugin;

impl Plugin for AIMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                follow_flag,
                roam,
                walk_into_range,
                walk_in_direction,
                reposition,
            ),
        );
        app.add_observer(friendly_formation_unit_in_front);
        app.add_observer(friendly_unit_in_front);
        app.add_observer(formation_has_target);
    }
}

const MOVE_EPSILON: f32 = 1.;

fn can_reposition_with(target_type: UnitType, other_type: UnitType) -> bool {
    match target_type {
        UnitType::Shieldwarrior => other_type == UnitType::Shieldwarrior,
        UnitType::Archer => true,
        UnitType::Pikeman => other_type == UnitType::Shieldwarrior,
        UnitType::Commander => false,
        UnitType::Bandit => false,
    }
}

fn follow_flag(
    query: Query<&BehaveCtx, With<FollowFlag>>,
    mut unit: Query<(
        &mut Velocity,
        &Transform,
        &FollowOffset,
        &RandomVelocityMul,
        &Speed,
        &FlagAssignment,
        &Unit,
        Option<&Target>,
    )>,
    is_attached: Query<&AttachedTo>,
    transform_query: Query<&Transform>,
) -> Result {
    for ctx in query.iter() {
        let (
            mut velocity,
            transform,
            offset,
            rand_velocity_mul,
            speed,
            flag_assignment,
            unit,
            has_taget,
        ) = unit.get_mut(ctx.target_entity())?;

        if has_taget.is_some() {
            return Ok(());
        }

        let flag_pos = transform_query
            .get(**flag_assignment)
            .unwrap()
            .translation
            .truncate();

        let target = match (
            is_attached.get(**flag_assignment).is_ok(),
            unit.unit_type.eq(&UnitType::Commander),
        ) {
            (true, true) | (true, false) | (false, false) => flag_pos + **offset,
            (false, true) => flag_pos,
        };

        let direction = (target.x - transform.translation.x).signum();

        if (transform.translation.x - target.x).abs() <= MOVE_EPSILON {
            velocity.0.x = 0.;
            continue;
        }

        velocity.0.x = direction * **speed * **rand_velocity_mul;
    }

    Ok(())
}

fn roam(
    time: Res<Time>,
    query: Query<&BehaveCtx, With<Roam>>,
    mut unit: Query<(&mut Roam, &mut Velocity, &RandomVelocityMul, &Speed)>,
) -> Result {
    for ctx in query.iter() {
        let (mut roam, mut velocity, rand_velocity_mul, speed) =
            unit.get_mut(ctx.target_entity())?;
        roam.tick(time.delta());

        if roam.just_finished() {
            let x = (fastrand::f32() - 0.5) * 2.;
            velocity.0.x = x.signum() * **speed * **rand_velocity_mul;
        }
    }
    Ok(())
}

fn walk_into_range(
    query: Query<&BehaveCtx, With<WalkIntoRange>>,
    mut unit: Query<(
        &mut Velocity,
        &Transform,
        Option<&Target>,
        &Speed,
        Option<&ProjectileRange>,
        Option<&MeleeRange>,
    )>,
    transform_query: Query<&Transform>,
    mut commands: Commands,
) -> Result {
    for ctx in query.iter() {
        let (mut velocity, transform, maybe_target, speed, projectile_range, melee_range) =
            unit.get_mut(ctx.target_entity())?;

        let Some(target) = maybe_target else {
            commands.trigger(ctx.failure());
            continue;
        };

        let target = transform_query.get(**target)?.translation.truncate();

        let range = if let Some(r) = projectile_range {
            **r
        } else if let Some(r) = melee_range {
            **r
        } else {
            commands.trigger(ctx.failure());
            continue;
        };

        let direction = (target.x - transform.translation.x).signum();
        if (transform.translation.x - target.x).abs() <= range {
            velocity.0.x = 0.;
            commands.trigger(ctx.failure());
            continue;
        }

        velocity.0.x = direction * **speed;
        commands.trigger(ctx.success());
    }

    Ok(())
}

fn friendly_formation_unit_in_front(
    trigger: On<BehaveTrigger<IsFriendlyFormationUnitInFront>>,
    units: Query<(&Unit, Option<&ArmyFormationTo>)>,
    other_units: Query<(Entity, &Transform, &Velocity, &BoxCollider, &Unit, &Name)>,
    commander: Query<&ArmyFormations>,
    has_target: Query<&RepositionTo>,
    time: Res<Time>,
    mut commands: Commands,
) -> Result {
    let target = trigger.ctx().target_entity();
    let ctx = trigger.event().ctx();

    if has_target.get(target).is_ok() {
        commands.trigger(ctx.failure());
        return Ok(());
    };
    // Get army formation for target
    let (target_unit, army_formation) = units.get(target)?;

    let army_formation = match army_formation {
        Some(formation) => formation,
        None => {
            commands.trigger(ctx.failure());
            return Ok(());
        }
    };

    // Get all formation units and collect with their unit types for sorting
    let entities = commander.get(**army_formation)?;
    let mut sorted_entities: Vec<Entity> = entities
        .iter()
        .filter(|entity| {
            if let Ok((_, _, _, _, unit, _)) = other_units.get(*entity) {
                return unit.unit_type == target_unit.unit_type;
            }
            false
        })
        .collect();

    // Find target's index in sorted list
    let target_idx = sorted_entities.iter().position(|e| e == &target);

    for entity in sorted_entities.iter() {
        if *entity == target {
            continue;
        }
        if let (
            Ok((_, target_transform, vel_target, box_1, target_unit, name_target)),
            Ok((_, other_transform, vel_other, box_2, other_unit, name_other)),
        ) = (other_units.get(target), other_units.get(*entity))
        {
            // Check if target unit type can reposition with other unit type

            // For same unit types, only allow reposition if target is earlier in sorted list
            if let (Some(t_idx), Some(other_idx)) =
                (target_idx, sorted_entities.iter().position(|e| e == entity))
            {
                if t_idx < other_idx {
                    info!(
                        "Skipping repositioning for unit {} as it is later in sorted list",
                        name_target
                    );
                    continue;
                }
            }

            let future_target =
                target_transform.translation.truncate() + vel_target.0 * time.delta_secs();
            let future_target_bound = box_1.at_pos(future_target);

            let future_other =
                other_transform.translation.truncate() + vel_other.0 * time.delta_secs();
            let future_other_bound = box_2.at_pos(future_other);

            if future_target_bound.intersects(&future_other_bound) {
                info!(
                    "Collision detected between units {} and {}",
                    name_target, name_other
                );
                let separation = box_1.half_size().x + box_2.half_size().x;

                commands.entity(target).insert(RepositionTo {
                    x_pos: other_transform.translation.x - separation,
                });

                commands.trigger(ctx.success());
                return Ok(());
            } else {
                commands.entity(target).remove::<RepositionTo>();
                commands.trigger(ctx.failure());
            }
        } else {
            // Unit in front not found or doesn't have required components
            commands.trigger(ctx.failure());
        }
    }

    commands.trigger(ctx.failure());

    Ok(())
}

fn friendly_unit_in_front(
    trigger: On<BehaveTrigger<IsFriendlyUnitInFront>>,
    units: Query<(&Unit, Option<&ArmyFormationTo>)>,
    other_units: Query<(Entity, &Transform, &Unit, &BoxCollider, &Name)>,
    commander: Query<&ArmyFormations>,
    mut commands: Commands,
) -> Result {
    let target = trigger.ctx().target_entity();
    let ctx = trigger.event().ctx();

    // Get army formation for target
    let (target_unit, army_formation) = units.get(target)?;
    if target_unit.unit_type.eq(&UnitType::Shieldwarrior) {
        commands.trigger(ctx.failure());
        return Ok(());
    }

    let army_formation = match army_formation {
        Some(formation) => formation,
        None => {
            commands.trigger(ctx.failure());
            return Ok(());
        }
    };

    let entities = commander.get(**army_formation)?;

    let (_, target_transform, _, target_box, target_name) = other_units.get(target)?;
    let mut furthest_back_x: Option<f32> = None;

    for entity in entities.iter() {
        if entity == target {
            continue;
        }

        if let Ok((_, other_transform, unit, other_box, other_name)) = other_units.get(entity) {
            let is_behind = target_transform.translation.x < other_transform.translation.x;

            // Unit is in front and should be behind - calculate repositioning target
            if !is_behind
                && ((target_unit.unit_type == UnitType::Archer
                    && (unit.unit_type == UnitType::Pikeman
                        || unit.unit_type == UnitType::Shieldwarrior))
                    || (target_unit.unit_type == UnitType::Pikeman
                        && unit.unit_type == UnitType::Shieldwarrior))
            {
                let separation = target_box.half_size().x + other_box.half_size().x;
                let target_x = other_transform.translation.x - separation - 12.;
                info!(
                    "target unit {:?} should be behind {}",
                    target_name, other_name
                );
                // Update to furthest back (lowest x value)
                furthest_back_x = Some(match furthest_back_x {
                    None => target_x,
                    Some(current) => current.min(target_x),
                });
            }
        }
    }

    if let Some(x_pos) = furthest_back_x {
        commands.entity(target).insert(RepositionTo { x_pos });
        commands.trigger(ctx.success());
    } else {
        commands.trigger(ctx.failure());
    }

    Ok(())
}

fn formation_has_target(
    trigger: On<BehaveTrigger<FormationHasTarget>>,
    has_target: Query<&Target>,
    formation_to_query: Query<(&ArmyFormationTo, &Name)>,
    commander: Query<&ArmyFormations>,
    mut commands: Commands,
) -> Result {
    let ctx = trigger.event().ctx();
    let target_entity = ctx.target_entity();

    let (formation_to, name) = formation_to_query.get(target_entity)?;

    if has_target.get(target_entity).is_ok() {
        commands.trigger(ctx.failure());
        return Ok(());
    }

    let formation = commander.get(**formation_to)?;

    let any_in_formation_has_target = formation
        .iter()
        .any(|entity| has_target.get(entity).is_ok());

    if any_in_formation_has_target {
        commands.trigger(ctx.success());
    } else {
        commands.trigger(ctx.failure());
    }
    Ok(())
}

fn reposition(
    query: Query<&BehaveCtx, With<Reposition>>,
    mut unit: Query<(&mut Velocity, &Speed, &Transform, &RepositionTo, &Name)>,
    mut commands: Commands,
) -> Result {
    for ctx in query.iter() {
        let (mut velocity, speed, transform, reposition_to, name) =
            unit.get_mut(ctx.target_entity())?;

        if (reposition_to.x_pos - transform.translation.x).abs() > 0.2 {
            let direction = reposition_to.x_pos.signum();
            info!(
                "Repositioning {} to x: {} at {}",
                name, reposition_to.x_pos, transform.translation.x
            );
            velocity.0.x = direction * **speed;
            commands.trigger(ctx.success());
        } else {
            commands
                .entity(ctx.target_entity())
                .remove::<RepositionTo>();
            commands.trigger(ctx.failure());
        }
    }

    Ok(())
}

fn walk_in_direction(
    query: Query<&BehaveCtx, With<WalkingInDirection>>,
    mut unit: Query<(
        &mut Velocity,
        &RandomVelocityMul,
        &Speed,
        Option<&ArmyFormationTo>,
    )>,
    has_target: Query<Option<&Target>>,
    commander: Query<&ArmyFormations>,
    target_location: Query<&Transform>,
) -> Result {
    for ctx in query.iter() {
        let (_, _, _, has_formation) = unit.get_mut(ctx.target_entity())?;

        let army_formation = match has_formation {
            Some(formation) => formation,
            None => {
                return Ok(());
            }
        };

        let formation = commander.get(**army_formation)?;

        let any_has_target = formation
            .iter()
            .find(|entity| has_target.get(*entity).is_ok());

        let unit_transform = target_location.get(ctx.target_entity())?;

        if let Some(target) = any_has_target {
            let target_transform = target_location.get(target)?;
            let direction = (target_transform.translation - unit_transform.translation)
                .signum()
                .x;

            if let Ok((mut entity_velocity, rand_velocity_mul, speed, _)) =
                unit.get_mut(ctx.target_entity())
            {
                entity_velocity.0.x = direction * **speed * **rand_velocity_mul;
            }
        }
    }
    Ok(())
}
