use bevy::prelude::*;

use super::{
    ai::AIPlugin,
    buildings::BuildingsPlugins,
    console::ConsolePlugin,
    entities::EntityPlugin,
    game_scenes::{GameScenesPlugin, start_game::StartGamePlugin},
    physics::PhysicsPlugin,
    players::PlayerPlugin,
};
use crate::networking::NetworkRegistry;

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            NetworkRegistry,
            AIPlugin,
            PhysicsPlugin,
            GameScenesPlugin,
            BuildingsPlugins,
            PlayerPlugin,
            EntityPlugin,
            ConsolePlugin,
        ));
    }
}
