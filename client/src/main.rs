use bevy::prelude::*;

use ai::AIPlugin;
use army::ArmyPlugins;
use bevy::audio::{AudioPlugin, SpatialScale, Volume};
use bevy_parallax::ParallaxPlugin;
use buildings::BuildingsPlugins;
use game_world::GameWorldPlugin;
use gizmos::GizmosPlugin;
use health::HealthPlugin;
use interactables::InteractablePlugins;
use interaction::InteractPlugin;
use items::ItemPlugins;
use lobby::LobbyPlugin;
use networking::join_server::JoinServerPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugins;
use projectiles::ProjectilePlugin;
use remote::CheatRemotePlugin;
use shared::PlayerState;
use shared::{GameState, SharedPlugin};
use std::env;
use ui::UiPlugin;
use units::UnitsPlugins;

use animations::AnimationPlugin;
use camera::CameraPlugin;
use entities::EntitiesPlugin;
use input::InputPlugin;
use travel::TravelPlugin;

use crate::{background::BackgroundPlugin, background_sound::BackgroundSoundPlugin};

pub mod background;
pub mod camera;
pub mod entities;
pub mod gizmos;
pub mod input;
pub mod networking;
pub mod ui;
pub mod widgets;

mod background_sound;

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

        let steam = aeronet_steam::steamworks::Client::init_app(1513980)
            .expect("failed to initialize steam");
        steam.networking_utils().init_relay_network_access();

        client
            .insert_resource(SteamworksClient(steam.clone()))
            .add_plugins(SteamworksPlugin::with(steam).unwrap());
    }

    let primary_window = Window {
        title: format!("WARPPC {user}"),
        resolution: (1280, 720).into(),
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
                global_volume: GlobalVolume::new(Volume::Linear(0.35)),
                default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
            }),
    );

    client.add_plugins(SharedPlugin);

    client
        .insert_state(GameState::Loading)
        .insert_state(PlayerState::World)
        .add_plugins((
            ParallaxPlugin,
            CameraPlugin,
            InputPlugin,
            AnimationPlugin,
            BackgroundPlugin,
            EntitiesPlugin,
            UiPlugin,
            BackgroundSoundPlugin,
            GizmosPlugin,
            GameWorldPlugin,
            TravelPlugin,
        ))
        .add_plugins((
            PlayerPlugins,
            BuildingsPlugins,
            ArmyPlugins,
            InteractPlugin,
            PhysicsPlugin,
            UnitsPlugins,
            HealthPlugin,
            AIPlugin,
            ProjectilePlugin,
            CheatRemotePlugin,
            InteractablePlugins,
            ItemPlugins,
            LobbyPlugin,
        ));

    client.add_systems(OnEnter(GameState::MainMenu), setup_background);

    if args.contains(&String::from("server")) {
        #[cfg(feature = "steam")]
        {
            use aeronet_steam::server::SteamNetServerPlugin;
            use lobby::create_server::create_steam_server;

            client
                .add_plugins(SteamNetServerPlugin)
                .add_systems(OnEnter(GameState::MainMenu), create_steam_server);
        }

        #[cfg(feature = "netcode")]
        {
            use lobby::create_server::create_web_transport_server;

            client.add_systems(OnEnter(GameState::MainMenu), create_web_transport_server);
        }
    } else {
        client.add_plugins(JoinServerPlugin);

        #[cfg(feature = "steam")]
        {
            use aeronet_steam::client::SteamNetClientPlugin;

            client.add_plugins(SteamNetClientPlugin);
        }

        #[cfg(feature = "netcode")]
        {
            use networking::join_server::join_web_transport_server;

            client.add_systems(OnEnter(GameState::MainMenu), join_web_transport_server);
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
        Mesh2d(meshes.add(Rectangle::new(90000.0, 2000.0))),
        MeshMaterial2d(materials.add(Color::hsl(332., 0.30, 0.17))),
        Transform::from_xyz(0.0, -1000.0, -1.0),
    ));
}
