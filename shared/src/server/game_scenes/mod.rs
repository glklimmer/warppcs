use bevy::prelude::*;

use map::MapPlugin;
use start_game::StartGamePlugin;
use travel::TravelPlugin;

pub mod map;
pub mod start_game;
pub mod travel;

pub struct GameScenesPlugin;

impl Plugin for GameScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((StartGamePlugin, MapPlugin, TravelPlugin));
    }
}
