use bevy::prelude::*;

use crate::networking::MultiplayerRoles;
use crate::server::ai::MOVE_EPSILON;
use crate::server::ai::{attack::unit_speed, UnitBehaviour};
use crate::server::entities::Unit;
use crate::GameState;
use crate::{networking::PlayerInput, BoxCollider, GRAVITY_G};

#[derive(Debug, Default, Component, Copy, Clone)]
pub struct Velocity(pub Vec2);

const PLAYER_MOVE_SPEED: f32 = 200.0;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (apply_velocity, determine_unit_velocity, apply_gravity).run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );

        app.add_systems(
            FixedUpdate,
            move_players_system.run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
    }
}

fn apply_gravity(mut query: Query<(&mut Velocity, &Transform, &BoxCollider)>, time: Res<Time>) {
    for (mut velocity, transform, collider) in &mut query {
        if transform.translation.y - collider.0.y > 0. {
            velocity.0.y -= GRAVITY_G * time.delta_seconds();
        } else {
            velocity.0.y = 0.;
        }
    }
}

fn move_players_system(mut query: Query<(&PlayerInput, &mut Velocity)>) {
    for (input, mut velocity) in query.iter_mut() {
        let x = (input.right as i8 - input.left as i8) as f32;
        let direction = Vec2::new(x, 0.).normalize_or_zero();
        velocity.0 = direction * PLAYER_MOVE_SPEED;
    }
}

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.) * time.delta_seconds();
    }
}

fn determine_unit_velocity(
    mut query: Query<(&mut Velocity, &Transform, &UnitBehaviour, &Unit)>,
    flag_transform: Query<&Transform, Without<Unit>>,
) {
    for (mut velocity, transform, behaviour, unit) in &mut query {
        match behaviour {
            UnitBehaviour::Idle | UnitBehaviour::AttackTarget(_) => velocity.0.x = 0.,
            UnitBehaviour::MoveTarget(target) => {
                set_velocity_with_target(target, transform, &mut velocity, unit);
            }
            UnitBehaviour::FollowFlag(flag, offset) => {
                let target = flag_transform.get(*flag).unwrap().translation.truncate();
                let target = target + *offset;
                set_velocity_with_target(&target, transform, &mut velocity, unit);
            }
        }
    }
}

fn set_velocity_with_target(
    target: &Vec2,
    transform: &Transform,
    velocity: &mut Mut<Velocity>,
    unit: &Unit,
) {
    if transform.translation.truncate().distance(*target) <= MOVE_EPSILON {
        velocity.0.x = 0.;
        return;
    }

    let target_right = target.x > transform.translation.x;

    match target_right {
        true => velocity.0.x = unit_speed(&unit.unit_type),
        false => velocity.0.x = -unit_speed(&unit.unit_type),
    }
}
