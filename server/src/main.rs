use bevy::prelude::*;

use bevy::app::ScheduleRunnerPlugin;
use bevy_renet::steam::SteamServerPlugin;
use shared::{
    SharedPlugin,
    server::{create_server::create_steam_server, networking::ServerNetworkPlugin},
    steamworks::SteamworksPlugin,
};

use std::time::Duration;

fn main() {
    let mut app = App::new();
    app.add_plugins(SteamworksPlugin::init_app(1513980).unwrap());

    app.add_plugins((
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
        SharedPlugin,
    ));

    app.add_plugins(ServerNetworkPlugin);
    app.add_plugins(SteamServerPlugin);

    app.add_systems(Startup, create_steam_server);

    app.run();
}
