use bevy::prelude::*;

use crate::networking::GameState;
use crate::server::ai::{attack::unit_speed, UnitBehaviour};
use crate::{
    networking::{PlayerInput, Unit},
    BoxCollider, GRAVITY_G,
};

#[derive(Debug, Default, Component, Copy, Clone)]
pub struct Velocity(pub Vec2);

const PLAYER_MOVE_SPEED: f32 = 200.0;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (apply_velocity, determine_unit_velocity, apply_gravity)
                .run_if(in_state(GameState::GameSession)),
        );

        app.add_systems(OnEnter(GameState::GameSession), move_players_system);
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

fn determine_unit_velocity(mut query: Query<(&mut Velocity, &Transform, &UnitBehaviour, &Unit)>) {
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
