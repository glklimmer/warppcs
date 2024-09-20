use bevy::prelude::*;

use ai::AIPlugin;
use movement::MovementPlugin;
use networking::ServerNetworkPlugin;

pub mod ai;
pub mod movement;
pub mod networking;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ServerNetworkPlugin);
        app.add_plugins(MovementPlugin);
        app.add_plugins(AIPlugin);
    }
}
