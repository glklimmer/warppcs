use bevy::prelude::*;

use super::{console::ConsolePlugin, create_server::CreateServerPlugin};

use crate::networking::NetworkRegistry;

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CreateServerPlugin, NetworkRegistry, ConsolePlugin));
    }
}
