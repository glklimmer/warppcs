use bevy::{math::bounding::IntersectsVolume, prelude::*};

use bevy_behave::prelude::{BehaveCtx, BehaveTrigger};

use super::{FollowOffset, Target, WalkIntoRange, WalkingInDirection};

use crate::{
    BoxCollider, Player,
    networking::UnitType,
    server::{
        buildings::recruiting::FlagAssignment,
        console::{ArmyFormationTo, ArmyFormations},
        entities::{MeleeRange, Unit},
        physics::{
            attachment::AttachedTo,
            movement::{RandomVelocityMul, Speed, Velocity},
        },
    },
};

#[derive(Component, Clone)]
pub struct FollowFlag;

#[derive(Component, Clone)]
pub struct IsFriendlyUnitInFront;

#[derive(Component, Clone, Deref, DerefMut)]
pub struct Roam(Timer);

impl Default for Roam {
    fn default() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Repeating))
    }
}

pub struct AIMovementPlugin;

impl Plugin for AIMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (follow_flag, roam, walk_into_range, walk_in_direction),
        );
        app.add_observer(friendly_unit_in_front);
    }
}

const MOVE_EPSILON: f32 = 1.;

fn follow_flag(
    query: Query<&BehaveCtx, With<FollowFlag>>,
    mut unit: Query<
        (
            &mut Velocity,
            &Transform,
            &FollowOffset,
            &RandomVelocityMul,
            &Speed,
            &FlagAssignment,
            &Unit,
            Option<&Target>,
        ),
        Without<Player>,
    >,
    is_attached: Query<&AttachedTo>,
    transform_query: Query<&Transform>,
) -> Result {
    for ctx in query.iter() {
        let (
            mut velocity,
            transform,
            follow_offset,
            rand_velocity_mul,
            speed,
            flag_assignment,
            unit,
            has_taget,
        ) = unit.get_mut(ctx.target_entity())?;

        if has_taget.is_some() {
            info!("Unit has a target");
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
            (true, true) | (true, false) | (false, false) => flag_pos + **follow_offset,
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
    mut query: Query<(&BehaveCtx, &mut Roam)>,
    mut unit: Query<(&mut Velocity, &RandomVelocityMul, &Speed)>,
    time: Res<Time>,
) -> Result {
    for (ctx, mut roam) in query.iter_mut() {
        (**roam).tick(time.delta());

        if !(**roam).just_finished() {
            continue;
        }

        if fastrand::f32() > 0.02 {
            continue;
        }

        let (mut velocity, rand_velocity_mul, speed) = unit.get_mut(ctx.target_entity())?;
        let choice = fastrand::u8(0..3);
        match choice {
            0 => {
                velocity.0.x = 0.0;
            }
            1 => {
                velocity.0.x = -1.0 * **speed * **rand_velocity_mul;
            }
            _ => {
                velocity.0.x = 1.0 * **speed * **rand_velocity_mul;
            }
        }
    }
    Ok(())
}

#[allow(clippy::type_complexity)]
fn walk_into_range(
    query: Query<&BehaveCtx, With<WalkIntoRange>>,
    mut unit: Query<(
        &mut Velocity,
        &Transform,
        Option<&Target>,
        &RandomVelocityMul,
        &Speed,
        &MeleeRange,
    )>,
    transform_query: Query<&Transform>,
    mut commands: Commands,
) -> Result {
    for ctx in query.iter() {
        let (mut velocity, transform, maybe_target, rand_velocity_mul, speed, range) =
            unit.get_mut(ctx.target_entity())?;

        let Some(target) = maybe_target else {
            commands.trigger(ctx.failure());
            continue;
        };

        let target = transform_query.get(**target)?.translation.truncate();

        let direction = (target.x - transform.translation.x).signum();
        info!("direction {}", direction);
        if (transform.translation.x - target.x).abs() <= **range {
            velocity.0.x = 0.;
            commands.trigger(ctx.success());
            continue;
        }

        velocity.0.x = direction * **speed * **rand_velocity_mul;
    }
    Ok(())
}

fn friendly_unit_in_front(
    trigger: On<BehaveTrigger<IsFriendlyUnitInFront>>,
    units: Query<(&Unit, Option<&ArmyFormationTo>)>,
    mut other_units: Query<(Entity, &Transform, &Unit, &BoxCollider)>,
    commander: Query<&ArmyFormations>,
    mut commands: Commands,
) -> Result {
    let target = trigger.ctx().target_entity();
    let ctx = trigger.event().ctx();

    // Get army formation for target
    let (unit, army_formation) = units.get(target)?;

    let army_formation = match army_formation {
        Some(formation) => formation,
        None => {
            commands.trigger(ctx.failure());
            return Ok(());
        }
    };

    // Get all formation units
    let entities = commander.get(**army_formation)?;
    let position = entities.iter().position(|entity| entity == target);

    match position {
        Some(pos) => {
            if pos == 0 {
                commands.trigger(ctx.failure());
                return Ok(());
            } else {
                // Get the unit in front (previous position in formation)
                let unit_in_front = entities.iter().nth(pos - 1).unwrap();

                // Get target and front unit data
                if let (Ok((_, transform_1, _, box_1)), Ok((_, transform_2, _, box_2))) =
                    (other_units.get(target), other_units.get(unit_in_front))
                {
                    // Check for collision using Bevy's intersects method
                    if box_1.at(transform_1).intersects(&box_2.at(transform_2)) {
                        commands.trigger(ctx.success());
                    } else {
                        commands.trigger(ctx.failure());
                    }
                } else {
                    // Unit in front not found or doesn't have required components
                    commands.trigger(ctx.failure());
                }
            }
        }
        None => {
            commands.trigger(ctx.failure());
            return Ok(());
        }
    }

    Ok(())
}

fn walk_in_direction(
    query: Query<(&BehaveCtx, &WalkingInDirection)>,
    mut unit: Query<(&mut Velocity, &RandomVelocityMul, &Speed)>,
) -> Result {
    for (ctx, walk) in query.iter() {
        let (mut velocity, rand_velocity_mul, speed) = unit.get_mut(ctx.target_entity())?;

        let direction: f32 = (**walk).into();

        velocity.0.x = direction * **speed * **rand_velocity_mul;
    }
    Ok(())
}
