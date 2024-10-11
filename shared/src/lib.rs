use bevy::prelude::*;

pub mod map;
pub mod networking;
pub mod server;
pub mod steamworks;

pub const GRAVITY_G: f32 = 9.81 * 100.;

#[derive(Component, Copy, Clone)]
pub struct BoxCollider(pub Vec2);

impl BoxCollider {
    pub fn half_size(&self) -> Vec2 {
        Vec2::new(self.0.x / 2., self.0.y / 2.)
    }
}
