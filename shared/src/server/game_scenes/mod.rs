use bevy::prelude::*;

use start_game::StartGamePlugin;
use travel::travel_player;

use crate::map::scenes::GameSceneId;

pub mod start_game;
pub mod travel;

#[derive(Component, Clone)]
pub struct GameSceneDestination {
    pub scene: GameSceneId,
    pub position: Vec3,
}

pub struct GameScenesPlugin;

impl Plugin for GameScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StartGamePlugin);

        app.add_systems(FixedUpdate, travel_player);
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Slot;
