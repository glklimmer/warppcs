use bevy::prelude::*;

use std::f32::consts::PI;

use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use animation::AnimationPlugin;
use bevy_renet::client_connected;
use camera::CameraPlugin;
use input::InputPlugin;
use king::KingPlugin;
use networking::{
    join_server::{join_own_server, join_server},
    ClientNetworkingPlugin, Connected,
};
use renet_steam::bevy::{SteamClientPlugin, SteamServerPlugin, SteamTransportError};
use shared::{
    networking::MultiplayerRoles,
    server::{create_server::create_server, networking::ServerNetworkPlugin},
    steamworks::SteamworksPlugin,
    GameState,
};
use ui::{MainMenuStates, MenuPlugin};

pub mod animation;
pub mod camera;
pub mod gizmos;
pub mod input;
pub mod king;
pub mod networking;
pub mod ui;
pub mod ui_widgets;

fn main() {
    let mut app = App::new();
    app.add_plugins(SteamworksPlugin::init_app(1513980).unwrap());

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));

    app.insert_state(GameState::MainMenu);
    app.insert_state(MultiplayerRoles::NotInGame);
    app.insert_state(MainMenuStates::TitleScreen);

    app.add_plugins(KingPlugin);
    app.add_plugins(CameraPlugin);
    app.add_plugins(InputPlugin);
    app.add_plugins(AnimationPlugin);
    app.add_plugins(MenuPlugin);

    app.add_systems(Startup, setup_background);

    app.add_plugins(ClientNetworkingPlugin);

    #[cfg(feature = "steam")]
    {
        app.add_plugins(ServerNetworkPlugin);
        app.add_plugins(SteamClientPlugin);
        app.add_plugins(SteamServerPlugin);

        app.configure_sets(Update, Connected.run_if(client_connected));

        //If any error is found we just panic
        #[allow(clippy::never_loop)]
        fn panic_on_error_system(mut renet_error: EventReader<SteamTransportError>) {
            for e in renet_error.read() {
                panic!("{}", e);
            }
        }

        app.add_systems(Update, panic_on_error_system.run_if(client_connected));
    }

    app.add_systems(
        OnEnter(MultiplayerRoles::Host),
        (create_server, join_own_server).chain(),
    );

    app.add_systems(Update, join_server.run_if(in_state(GameState::MainMenu)));

    app.run();
}

fn setup_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Plain
    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(6000.0, 2000.0))),
        material: materials.add(Color::hsl(109., 0.97, 0.88)),
        transform: Transform::from_xyz(0.0, -1000.0, 0.0),
        ..default()
    });

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}
