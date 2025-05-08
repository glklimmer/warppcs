use bevy::prelude::*;

use start_game::StartGamePlugin;
use travel::TravelPlugin;

pub mod start_game;
pub mod travel;

pub struct GameScenesPlugin;

impl Plugin for GameScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((StartGamePlugin, TravelPlugin));
    }
}
