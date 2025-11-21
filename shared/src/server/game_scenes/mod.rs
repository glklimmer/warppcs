use bevy::prelude::*;

use init_world::StartGamePlugin;
use travel::TravelPlugin;
use world::WorldPlugin;

pub mod init_world;
pub mod travel;
pub mod world;

#[derive(Component, PartialEq, Eq, Copy, Clone)]
pub struct GameSceneId(usize);
impl GameSceneId {
    pub(crate) fn lobby() -> Self {
        Self(0)
    }
}

pub struct GameScenesPlugin;

impl Plugin for GameScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((StartGamePlugin, WorldPlugin, TravelPlugin));
    }
}
