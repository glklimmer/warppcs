use bevy::prelude::*;

use bevy::audio::{AudioPlugin, SpatialScale, Volume};
use bevy_parallax::ParallaxPlugin;
use gizmos::GizmosPlugin;
use map::MapPlugin;
use networking::join_server::{JoinServerPlugin, join_web_transport_server};
use shared::server::create_server::{create_steam_server, create_web_transport_server};
use shared::{
    GameState, SharedPlugin, networking::NetworkRegistry, server::networking::ServerNetworkPlugin,
};
use std::env;
use ui::UiPlugin;
use widgets::WidgetsPlugin;

use animations::AnimationPlugin;
use camera::CameraPlugin;
use entities::EntitiesPlugin;
use input::InputPlugin;
use sound::SoundPlugin;

pub mod animations;
pub mod camera;
pub mod entities;
pub mod gizmos;
pub mod input;
pub mod map;
pub mod networking;
pub mod sound;
pub mod ui;
pub mod widgets;

/// Spatial audio uses the distance to attenuate the sound volume. In 2D with the default camera,
/// 1 pixel is 1 unit of distance, so we use a scale so that 100 pixels is 1 unit of distance for
/// audio.
const AUDIO_SCALE: f32 = 1. / 200.0;

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
        use aeronet_steam::SteamworksClient;
        use bevy_steamworks::SteamworksPlugin;

        let (steam, steam_single) = aeronet_steam::steamworks::Client::init_app(1513980)
            .expect("failed to initialize steam");
        steam.networking_utils().init_relay_network_access();

        client
            .insert_resource(SteamworksClient(steam.clone()))
            .insert_non_send_resource(steam_single)
            .add_plugins(SteamworksPlugin::with(steam).unwrap());
    }

    let primary_window = Window {
        title: format!("WARPPC {}", user),
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
            .set(ImagePlugin::default_nearest())
            .set(AudioPlugin {
                global_volume: GlobalVolume::new(Volume::Linear(0.)),
                default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
            }),
    );

    client.add_plugins(SharedPlugin);

    client.insert_state(GameState::MainMenu).add_plugins((
        ParallaxPlugin,
        CameraPlugin,
        InputPlugin,
        AnimationPlugin,
        // MenuPlugin,
        EntitiesPlugin,
        WidgetsPlugin,
        UiPlugin,
        MapPlugin,
        SoundPlugin,
        GizmosPlugin,
    ));

    client.add_systems(Startup, setup_background);

    if args.contains(&String::from("server")) {
        client.add_plugins(ServerNetworkPlugin);

        #[cfg(feature = "steam")]
        {
            use aeronet_steam::server::SteamNetServerPlugin;
            use bevy_steamworks::ClientManager;

            client
                .add_plugins(SteamNetServerPlugin::<ClientManager>::default())
                .add_systems(Startup, create_steam_server);
        }

        #[cfg(feature = "netcode")]
        {
            client.add_systems(Startup, create_web_transport_server);
        }
    } else {
        client.add_plugins((NetworkRegistry, JoinServerPlugin));

        #[cfg(feature = "steam")]
        {
            use aeronet_steam::client::SteamNetClientPlugin;
            use bevy_steamworks::ClientManager;

            client.add_plugins(SteamNetClientPlugin::<ClientManager>::default());
        }

        #[cfg(feature = "netcode")]
        {
            client.add_systems(Startup, join_web_transport_server);
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
