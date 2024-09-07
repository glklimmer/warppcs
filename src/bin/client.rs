use bevy::prelude::*;

use warppcs::{
    client::{camera::CameraPlugin, input::InputPlugin, networking::ClientNetworkingPlugin},
    shared::networking::setup_level,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    app.add_plugins(ClientNetworkingPlugin);
    app.add_plugins(CameraPlugin);
    app.add_plugins(InputPlugin);

    app.add_systems(Startup, setup_level);

    app.run();
}
