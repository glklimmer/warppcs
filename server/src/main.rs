use bevy::prelude::*;

use bevy::app::ScheduleRunnerPlugin;
use shared::{
    server::{
        ai::AIPlugin, create_server::create_server, game_scenes::GameScenesPlugin,
        networking::ServerNetworkPlugin, physics::PhysicsPlugin,
    },
    steamworks::SteamworksPlugin,
};

use std::time::Duration;

fn main() {
    let mut app = App::new();
    app.add_plugins(SteamworksPlugin::init_app(1513980).unwrap());

    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
    );

    app.add_plugins(ServerNetworkPlugin);

    app.add_systems(Startup, create_server);

    app.run();
}
