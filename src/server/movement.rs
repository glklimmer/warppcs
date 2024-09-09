use bevy::prelude::*;

use crate::shared::networking::{Facing, Movement, PlayerInput};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, apply_velocity_system);

        app.add_systems(Update, move_players_system);
    }
}

#[derive(Debug, Default, Component)]
pub struct Velocity(pub Vec2);

const PLAYER_MOVE_SPEED: f32 = 200.0;

fn move_players_system(mut query: Query<(&PlayerInput, &Transform, &mut Velocity, &mut Movement)>) {
    for (input, transform, mut velocity, mut movement) in query.iter_mut() {
        let x = (input.right as i8 - input.left as i8) as f32;
        let direction = Vec2::new(x, 0.).normalize_or_zero();
        velocity.0 = direction * PLAYER_MOVE_SPEED;

        movement.translation = transform.translation.into();
        if input.right {
            movement.facing = Facing::Right
        }
        if input.left {
            movement.facing = Facing::Left
        }
        movement.moving = x != 0.;
    }
}

fn apply_velocity_system(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.) * time.delta_seconds();
    }
}
