use bevy::prelude::*;

use collider::ColliderPlugin;
use movement::MovementPlugin;

pub mod collider;
pub mod movement;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MovementPlugin);
        app.add_plugins(ColliderPlugin);
    }
}
