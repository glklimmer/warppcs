use std::cmp::Ordering;

use bevy::prelude::*;

use crate::shared::networking::{Facing, Movement, PlayerInput, UnitType};

#[derive(Debug, Default, Component)]
pub struct Velocity(pub Vec2);

const PLAYER_MOVE_SPEED: f32 = 200.0;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, apply_velocity_system);

        app.add_systems(
            Update,
            (
                move_players_system,
                unit_random_target,
                unit_move_towards_target,
            ),
        );
    }
}

fn move_players_system(mut query: Query<(&PlayerInput, &mut Velocity)>) {
    for (input, mut velocity) in query.iter_mut() {
        let x = (input.right as i8 - input.left as i8) as f32;
        let direction = Vec2::new(x, 0.).normalize_or_zero();
        velocity.0 = direction * PLAYER_MOVE_SPEED;
    }
}

fn apply_velocity_system(
    mut query: Query<(&Velocity, &mut Transform, &mut Movement)>,
    time: Res<Time>,
) {
    for (velocity, mut transform, mut movement) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.) * time.delta_seconds();
        movement.translation = transform.translation.into();

        if velocity.0.x > 0. {
            movement.facing = Facing::Right
        }
        if velocity.0.x < 0. {
            movement.facing = Facing::Left
        }
        movement.moving = velocity.0.x != 0.;
    }
}

#[derive(Component)]
struct MovementTarget(Vec3);

#[allow(clippy::type_complexity)]
fn unit_random_target(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&MovementTarget>), (With<UnitType>, With<Movement>)>,
) {
    for (entity, transform, target_option) in query.iter() {
        if target_option.is_none() {
            let mut new_target = transform.translation;
            new_target.x += (fastrand::f32() - 0.5) * 600.;

            commands.entity(entity).insert(MovementTarget(new_target));
        }
    }
}

const UNIT_MOVE_SPEED: f32 = 100.;

#[allow(clippy::type_complexity)]
fn unit_move_towards_target(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut Velocity, &Transform, &MovementTarget),
        (With<UnitType>, With<Movement>),
    >,
) {
    for (entity, mut velocity, transform, target) in &mut query {
        let delta_x = (transform.translation.x - target.0.x).abs();

        if delta_x <= 20.0 {
            velocity.0.x = 0.;
            commands.entity(entity).remove::<MovementTarget>();
        } else {
            match transform.translation.x.total_cmp(&target.0.x) {
                Ordering::Less => velocity.0.x = UNIT_MOVE_SPEED,
                Ordering::Greater => velocity.0.x = -UNIT_MOVE_SPEED,
                _ => {} // Not needed anymore as the delta_x check handles it
            }
        }
    }
}
