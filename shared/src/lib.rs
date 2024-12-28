use bevy::prelude::*;

pub mod enum_map;
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

#[derive(Component)]
struct DelayedDespawn(Timer);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    GameSession,
}
