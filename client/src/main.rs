use animations::AnimationsPlugin;
use bevy::prelude::*;

use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_renet::client_connected;
use camera::CameraPlugin;
use input::InputPlugin;
use networking::{ClientNetworkPlugin, Connected};
use shared::{networking::MultiplayerRoles, server::networking::ServerNetworkPlugin, GameState};
use std::f32::consts::PI;
use ui::{MainMenuStates, MenuPlugin};

pub mod animations;
pub mod camera;
pub mod gizmos;
pub mod input;
pub mod networking;
pub mod ui;
pub mod ui_widgets;

fn main() {
    let mut app = App::new();
    #[cfg(feature = "steam")]
    {
        use shared::steamworks::SteamworksPlugin;
        app.add_plugins(SteamworksPlugin::init_app(1513980).unwrap());
    }

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));

    app.insert_state(GameState::MainMenu);
    app.insert_state(MultiplayerRoles::NotInGame);
    app.insert_state(MainMenuStates::TitleScreen);

    app.add_plugins(CameraPlugin);
    app.add_plugins(InputPlugin);
    app.add_plugins(AnimationsPlugin);
    app.add_plugins(MenuPlugin);

    app.add_systems(Startup, setup_background);

    app.add_plugins(ServerNetworkPlugin);
    app.add_plugins(ClientNetworkPlugin);

    #[cfg(feature = "steam")]
    {
        use networking::join_server::{join_own_steam_server, join_steam_server};
        use renet_steam::bevy::{SteamClientPlugin, SteamServerPlugin, SteamTransportError};
        use shared::server::create_server::create_steam_server;

        app.add_plugins(SteamServerPlugin);
        app.add_plugins(SteamClientPlugin);

        app.configure_sets(Update, Connected.run_if(client_connected));

        #[allow(clippy::never_loop)]
        fn panic_on_error_system(mut renet_error: EventReader<SteamTransportError>) {
            for e in renet_error.read() {
                panic!("{}", e);
            }
        }

        app.add_systems(Update, panic_on_error_system.run_if(client_connected));

        app.add_systems(
            OnEnter(MultiplayerRoles::Host),
            (create_steam_server, join_own_steam_server).chain(),
        );

        app.add_systems(
            Update,
            join_steam_server.run_if(in_state(GameState::MainMenu)),
        );
    }

    #[cfg(feature = "netcode")]
    {
        use bevy_renet::transport::NetcodeClientPlugin;
        use bevy_renet::{renet::transport::NetcodeTransportError, transport::NetcodeServerPlugin};
        use networking::join_server::{join_netcode_server, join_own_netcode_server};
        use shared::server::create_server::create_netcode_server;

        app.add_plugins(NetcodeServerPlugin);
        app.add_plugins(NetcodeClientPlugin);

        app.configure_sets(Update, Connected.run_if(client_connected));

        #[allow(clippy::never_loop)]
        fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
            for e in renet_error.read() {
                panic!("{}", e);
            }
        }

        app.add_systems(Update, panic_on_error_system.run_if(client_connected));

        app.add_systems(
            OnEnter(MultiplayerRoles::Host),
            (create_netcode_server, join_own_netcode_server).chain(),
        );

        app.add_systems(
            Update,
            join_netcode_server.run_if(in_state(GameState::MainMenu)),
        );
    }

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
