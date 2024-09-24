use bevy::prelude::*;

use ai::AIPlugin;
use networking::ServerNetworkPlugin;
use physics::PhysicsPlugin;

pub mod ai;
pub mod networking;
pub mod physics;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ServerNetworkPlugin);
        app.add_plugins(AIPlugin);
        app.add_plugins(PhysicsPlugin);
    }
}
