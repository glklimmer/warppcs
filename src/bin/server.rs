use bevy::prelude::*;

use bevy::app::ScheduleRunnerPlugin;
use std::time::Duration;
use warppcs::server::{ai::AIPlugin, movement::MovementPlugin, networking::ServerNetworkPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
    );

    app.add_plugins(ServerNetworkPlugin);
    app.add_plugins(MovementPlugin);
    app.add_plugins(AIPlugin);

    app.run();
}
