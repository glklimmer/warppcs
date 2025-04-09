use bevy::prelude::*;

use bevy_parallax::ParallaxPlugin;
use bevy_renet::client_connected;
use gizmos::GizmosPlugin;
use menu::{MainMenuStates, MenuPlugin};
use networking::Connected;
use shared::{
    GameState, SharedPlugin, networking::NetworkRegistry, server::networking::ServerNetworkPlugin,
};
use std::env;
use ui::UiPlugin;

use animations::AnimationPlugin;
use camera::CameraPlugin;
use entities::EntitiesPlugin;
use input::InputPlugin;

#[cfg(feature = "steam")]
use bevy_renet::steam::{SteamClientPlugin, SteamTransportError};
#[cfg(feature = "steam")]
use menu::JoinSteamLobby;
#[cfg(feature = "steam")]
use networking::join_server::join_steam_server;

#[cfg(feature = "netcode")]
use bevy_renet::netcode::NetcodeTransportError;
#[cfg(feature = "netcode")]
use networking::join_server::join_netcode_server;
#[cfg(feature = "netcode")]
use shared::server::create_server::create_netcode_server;

pub mod animations;
pub mod camera;
pub mod entities;
pub mod gizmos;
pub mod input;
pub mod menu;
pub mod networking;
pub mod ui;
pub mod ui_widgets;

fn main() {
    let args: Vec<String> = env::args().collect();
    let user = if args.contains(&String::from("server")) {
        "server"
    } else {
        "client"
    };
    let mut client = App::new();

    #[cfg(feature = "steam")]
    {
        use shared::steamworks::SteamworksPlugin;
        client.add_plugins(SteamworksPlugin::init_app(1513980).unwrap());
    }

    client.add_plugins((DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("WARPPC {}", user),
                resolution: (1280.0, 720.0).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin::default_nearest()),));

    client.add_plugins(SharedPlugin);

    client
        .insert_state(MainMenuStates::TitleScreen)
        .insert_state(GameState::MainMenu)
        .add_plugins((
            ParallaxPlugin,
            CameraPlugin,
            InputPlugin,
            AnimationPlugin,
            MenuPlugin,
            EntitiesPlugin,
            UiPlugin,
            GizmosPlugin,
        ));

    client.add_systems(Startup, setup_background);

    #[cfg(feature = "steam")]
    {
        client.add_plugins(SteamClientPlugin);

        client.configure_sets(Update, Connected.run_if(client_connected));

        fn panic_on_error_system(mut renet_error: EventReader<SteamTransportError>) {
            #[allow(clippy::never_loop)]
            for e in renet_error.read() {
                panic!("{}", e);
            }
        }

        client.add_systems(Update, panic_on_error_system.run_if(client_connected));
        client.add_systems(Update, join_steam_server.run_if(on_event::<JoinSteamLobby>));
    }

    #[cfg(feature = "netcode")]
    {
        client.configure_sets(Update, Connected.run_if(client_connected));

        fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
            #[allow(clippy::never_loop)]
            for e in renet_error.read() {
                panic!("{}", e);
            }
        }

        client.add_systems(Update, panic_on_error_system.run_if(client_connected));

        if args.contains(&String::from("server")) {
            client
                .add_plugins(ServerNetworkPlugin)
                .add_systems(Startup, create_netcode_server);
        } else {
            client
                .add_plugins(NetworkRegistry)
                .add_systems(Startup, join_netcode_server);
        }
    }

    client.run();
}

fn setup_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Plain
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(60000.0, 2000.0))),
        MeshMaterial2d(materials.add(Color::hsl(109., 0.97, 0.88))),
        Transform::from_xyz(0.0, -1000.0, 0.0),
    ));
}
