use bevy::prelude::*;

use map::MapPlugin;
use start_game::StartGamePlugin;
use travel::TravelPlugin;

pub mod map;
pub mod start_game;
pub mod travel;

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
        app.add_plugins((StartGamePlugin, MapPlugin, TravelPlugin));
    }
}
