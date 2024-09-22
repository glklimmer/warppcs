use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};

use ai::AIPlugin;
use networking::ServerNetworkPlugin;
use physics::PhysicsPlugin;
use shared::SharedPlugin;

pub mod ai;
pub mod networking;
pub mod physics;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
    );

    app.add_plugins(ServerNetworkPlugin);
    app.add_plugins(AIPlugin);
    app.add_plugins(PhysicsPlugin);

    app.add_plugins(SharedPlugin);

    app.run();
}
