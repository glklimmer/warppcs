use bevy::prelude::*;

use bevy::app::ScheduleRunnerPlugin;
use bevy_parallax::ParallaxPlugin;
use bevy_renet::client_connected;
use core::time::Duration;
use gizmos::GizmosPlugin;
use menu::{MainMenuStates, MenuPlugin};
use networking::{ClientNetworkPlugin, Connected};
use shared::{server::networking::ServerNetworkPlugin, GameState};
use sound::SoundPlugin;
use std::env;
use std::f32::consts::PI;
use std::thread;
use ui::UiPlugin;

use animations::AnimationPlugin;
use camera::CameraPlugin;
use entities::{player::ClientPlayer, EntitiesPlugin};
use input::InputPlugin;

#[cfg(feature = "steam")]
use bevy_renet::steam::{SteamClientPlugin, SteamTransportError};
#[cfg(feature = "steam")]
use menu::JoinSteamLobby;
#[cfg(feature = "steam")]
use networking::join_server::join_steam_server;

#[cfg(feature = "netcode")]
use bevy_renet::netcode::{NetcodeClientPlugin, NetcodeServerPlugin, NetcodeTransportError};
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
pub mod sound;
pub mod ui;
pub mod ui_widgets;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from("server")) {
        thread::Builder::new()
            .name("server".into())
            .spawn(|| {
                let mut server = App::new();

                server.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                    Duration::from_secs_f64(1.0 / 60.0),
                )));

                server.add_plugins(ServerNetworkPlugin);

                println!("Starting netcode server...");

                #[cfg(feature = "netcode")]
                {
                    server.add_plugins(NetcodeServerPlugin);
                    server.add_systems(Startup, create_netcode_server);
                }

                server.run();
            })
            .unwrap();
    }

    let mut client = App::new();

    #[cfg(feature = "steam")]
    {
        use shared::steamworks::SteamworksPlugin;
        client.add_plugins(SteamworksPlugin::init_app(1513980).unwrap());
    }

    let primary_window = Window {
        title: "WARPPCS".to_string(),
        resolution: (1280.0, 720.0).into(),
        resizable: false,
        ..default()
    };

    client.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(primary_window),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    );

    client.insert_state(MainMenuStates::TitleScreen);
    client.insert_state(GameState::MainMenu);

    client.add_plugins(ParallaxPlugin);
    client.add_plugins(CameraPlugin);
    client.add_plugins(InputPlugin);
    client.add_plugins(AnimationPlugin);
    client.add_plugins(MenuPlugin);
    client.add_plugins(EntitiesPlugin);
    client.add_plugins(UiPlugin);
    client.add_plugins(GizmosPlugin);
    client.add_plugins(SoundPlugin);
    client.add_systems(Startup, setup_background);
    client.add_plugins(ClientNetworkPlugin);

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
        client.add_plugins(NetcodeClientPlugin);

        client.configure_sets(Update, Connected.run_if(client_connected));

        fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
            #[allow(clippy::never_loop)]
            for e in renet_error.read() {
                panic!("{}", e);
            }
        }

        client.add_systems(Update, panic_on_error_system.run_if(client_connected));
        client.add_systems(Startup, join_netcode_server);
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
        Mesh2d(meshes.add(Rectangle::new(6000.0, 2000.0))),
        MeshMaterial2d(materials.add(Color::hsl(109., 0.97, 0.88))),
        Transform::from_xyz(0.0, -1000.0, 0.0),
    ));

    // light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
    ));
}
