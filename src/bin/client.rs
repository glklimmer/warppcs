use bevy::{
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
};
use iyes_perf_ui::{entries::PerfUiBundle, PerfUiPlugin};
use warppcs::client::{gizmos::GizmosPlugin, ClientPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));

    app.add_plugins(ClientPlugin);

    // app.add_plugins(GizmosPlugin);

    // app.add_plugins(PerfUiPlugin);
    // app.add_systems(Startup, debug_system);
    // app.add_plugins(FrameTimeDiagnosticsPlugin);
    // app.add_plugins(EntityCountDiagnosticsPlugin);
    // app.add_plugins(SystemInformationDiagnosticsPlugin);
    // This shit break shit
    // app.add_plugins(FrameTimeDiagnosticsPlugin::default());

    app.run();
}

fn debug_system(mut commands: Commands) {
    commands.spawn(PerfUiBundle::default());
}
