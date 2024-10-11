use bevy::prelude::*;

use movement::MovementPlugin;
use projectile::ProjectilePlugin;

pub mod movement;
pub mod projectile;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MovementPlugin);
        app.add_plugins(ProjectilePlugin);
    }
}
