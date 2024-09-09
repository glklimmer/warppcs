use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use warppcs::{
    client::{
        animation::AnimationPlugin, camera::CameraPlugin, input::InputPlugin,
        movement::MovementPlugin, networking::ClientNetworkingPlugin,
    },
    shared::networking::setup_level,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    app.add_plugins(ClientNetworkingPlugin);
    app.add_plugins(CameraPlugin);
    app.add_plugins(InputPlugin);
    app.add_plugins(MovementPlugin);
    app.add_plugins(AnimationPlugin);

    app.add_systems(Startup, setup_level);

    app.add_plugins(FrameTimeDiagnosticsPlugin);

    app.run();
}
