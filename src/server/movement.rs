use bevy::prelude::*;

use crate::shared::networking::{Facing, Movement, PlayerInput, Unit};

use super::ai::{attack::unit_speed, UnitBehaviour};

#[derive(Debug, Default, Component)]
pub struct Velocity(pub Vec2);

const PLAYER_MOVE_SPEED: f32 = 200.0;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (apply_velocity_system, determine_velocity));

        app.add_systems(
            Update,
            (
                move_players_system,
                // unit_random_target,
                // unit_move_towards_target,
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

fn determine_velocity(mut query: Query<(&mut Velocity, &Transform, &UnitBehaviour, &Unit)>) {
    for (mut velocity, transform, behaviour, unit) in &mut query {
        match behaviour {
            UnitBehaviour::Idle | UnitBehaviour::AttackTarget(_) => velocity.0.x = 0.,
            UnitBehaviour::MoveTarget(target) => {
                let target_right = target.x > transform.translation.x;

                match target_right {
                    true => velocity.0.x = unit_speed(&unit.unit_type),
                    false => velocity.0.x = -unit_speed(&unit.unit_type),
                }
            }
        }
    }
}
