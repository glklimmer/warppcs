use bevy::prelude::*;

use bevy_renet::renet::ClientId;

use super::{
    ai::AIPlugin,
    buildings::BuildingsPlugins,
    entities::EntityPlugin,
    game_scenes::{GameScenesPlugin, start_game::StartGamePlugin},
    physics::PhysicsPlugin,
    players::PlayerPlugin,
};
use crate::networking::{NetworkRegistry, PlayerCommand};

#[derive(Event)]
pub struct NetworkEvent {
    pub client_id: ClientId,
    pub message: PlayerCommand,
}

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            NetworkRegistry,
            StartGamePlugin,
            AIPlugin,
            PhysicsPlugin,
            GameScenesPlugin,
            BuildingsPlugins,
            PlayerPlugin,
            EntityPlugin,
        ));
    }
}
