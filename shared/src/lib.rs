use bevy::prelude::*;

use castle::CastlePlugin;

pub mod castle;
pub mod networking;

pub const GRAVITY_G: f32 = 9.81 * 100.;

#[derive(Component)]
pub struct BoxCollider(pub Vec2);

impl BoxCollider {
    pub fn half_size(&self) -> Vec2 {
        Vec2::new(self.0.x / 2., self.0.y / 2.)
    }
}

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CastlePlugin);
    }
}
