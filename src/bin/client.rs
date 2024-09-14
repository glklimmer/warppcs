use bevy::{
    color::palettes::css::RED,
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
};
use iyes_perf_ui::{entries::PerfUiBundle, PerfUiPlugin};
use warppcs::{
    client::{
        animation::AnimationPlugin, camera::CameraPlugin, input::InputPlugin, king::KingPlugin,
        movement::MovementPlugin, networking::ClientNetworkingPlugin,
    },
    server::ai::attack::unit_range,
    shared::networking::{setup_level, Unit},
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));

    app.add_plugins(KingPlugin);
    app.add_plugins(ClientNetworkingPlugin);
    app.add_plugins(CameraPlugin);
    app.add_plugins(InputPlugin);
    app.add_plugins(MovementPlugin);
    app.add_plugins(AnimationPlugin);

    app.add_systems(Startup, setup_level);

    app.add_plugins(PerfUiPlugin);
    app.add_systems(Startup, debug_system);
    app.add_plugins(FrameTimeDiagnosticsPlugin);
    app.add_plugins(EntityCountDiagnosticsPlugin);
    app.add_plugins(SystemInformationDiagnosticsPlugin);
    // This shit break shit
    // app.add_plugins(FrameTimeDiagnosticsPlugin::default());

    app.add_systems(Update, draw_range);

    app.run();
}

fn debug_system(mut commands: Commands) {
    commands.spawn(PerfUiBundle::default());
}

fn draw_range(mut gizmos: Gizmos, query: Query<(&Transform, &Unit)>) {
    for (transform, unit) in query.iter() {
        gizmos.circle_2d(
            transform.translation.truncate(),
            unit_range(&unit.unit_type),
            RED,
        );
    }
}
