use bevy::prelude::*;

use super::{
    ai::AIPlugin, buildings::BuildingsPlugins, console::ConsolePlugin,
    create_server::CreateServerPlugin, entities::EntityPlugin, physics::PhysicsPlugin,
    players::PlayerPlugin,
};
use crate::networking::NetworkRegistry;

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CreateServerPlugin,
            NetworkRegistry,
            AIPlugin,
            PhysicsPlugin,
            BuildingsPlugins,
            PlayerPlugin,
            EntityPlugin,
            ConsolePlugin,
        ));
    }
}
