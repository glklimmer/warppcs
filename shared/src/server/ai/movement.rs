use bevy::prelude::*;

use bevy_behave::prelude::BehaveCtx;

use super::{FollowFlag, FollowOffset, Target, WalkIntoRange, WalkingInDirection};

use crate::server::{
    buildings::recruiting::FlagAssignment,
    entities::Range,
    physics::movement::{RandomVelocityMul, Speed, Velocity},
};

pub struct AIMovementPlugin;

impl Plugin for AIMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (follow_flag, walk_into_range, walk_in_direction),
        );
    }
}

const MOVE_EPSILON: f32 = 1.;

fn follow_flag(
    query: Query<&BehaveCtx, With<FollowFlag>>,
    mut unit: Query<(
        &mut Velocity,
        &Transform,
        &FollowOffset,
        &RandomVelocityMul,
        &Speed,
        &FlagAssignment,
    )>,
    transform_query: Query<&Transform>,
) {
    for ctx in query.iter() {
        let Ok((mut velocity, transform, follow_offset, rand_velocity_mul, speed, flag_assignment)) =
            unit.get_mut(ctx.target_entity())
        else {
            continue;
        };
        let flag_pos = transform_query
            .get(**flag_assignment)
            .unwrap()
            .translation
            .truncate();

        let target = flag_pos + **follow_offset;
        let direction = (target.x - transform.translation.x).signum();

        if (transform.translation.x - target.x).abs() <= MOVE_EPSILON {
            velocity.0.x = 0.;
            continue;
        }

        velocity.0.x = direction * **speed * **rand_velocity_mul;
    }
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
        &Range,
    )>,
    transform_query: Query<&Transform>,
    mut commands: Commands,
) {
    for ctx in query.iter() {
        let (mut velocity, transform, maybe_target, rand_velocity_mul, speed, range) =
            unit.get_mut(ctx.target_entity()).unwrap();

        let Some(target) = maybe_target else {
            commands.trigger(ctx.success());
            continue;
        };

        let target = transform_query
            .get(**target)
            .unwrap()
            .translation
            .truncate();

        let direction = (target.x - transform.translation.x).signum();

        if (transform.translation.x - target.x).abs() <= **range {
            velocity.0.x = 0.;
            commands.trigger(ctx.success());
            continue;
        }

        velocity.0.x = direction * **speed * **rand_velocity_mul;
    }
}

fn walk_in_direction(
    query: Query<(&BehaveCtx, &WalkingInDirection)>,
    mut unit: Query<(&mut Velocity, &RandomVelocityMul, &Speed)>,
) {
    for (ctx, walk) in query.iter() {
        let (mut velocity, rand_velocity_mul, speed) = unit.get_mut(ctx.target_entity()).unwrap();

        let direction: f32 = (**walk).into();

        velocity.0.x = direction * **speed * **rand_velocity_mul;
    }
}
