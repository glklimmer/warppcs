use bevy::prelude::*;

use std::f32::consts::PI;

use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use animation::AnimationPlugin;
use bevy_renet::{client_connected, renet::RenetClient};
use camera::CameraPlugin;
use input::InputPlugin;
use king::KingPlugin;
use networking::{ClientNetworkingPlugin, Connected, CurrentClientId};
use renet_steam::{
    bevy::{SteamClientPlugin, SteamServerPlugin, SteamTransportError},
    SteamClientTransport,
};
use shared::{
    networking::connection_config,
    server::{create_server::create_server, networking::ServerNetworkPlugin},
    steamworks::{SteamworksClient, SteamworksPlugin},
};
use steamworks::SteamId;
use ui::MenuPlugin;

pub mod animation;
pub mod camera;
pub mod gizmos;
pub mod input;
pub mod king;
pub mod networking;
pub mod ui;

fn main() {
    let mut app = App::new();
    app.add_plugins(SteamworksPlugin::init_app(1513980).unwrap());

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

        app.add_systems(Update, panic_on_error_system);
    }

    app.add_plugins(ClientNetworkingPlugin);

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));

    app.add_plugins(KingPlugin);
    app.add_plugins(CameraPlugin);
    app.add_plugins(InputPlugin);
    app.add_plugins(AnimationPlugin);
    app.add_plugins(MenuPlugin);

    app.add_systems(Startup, setup_background);
    app.add_systems(Startup, (create_server, join_server).chain());

    app.run();
}

fn join_server(mut commands: Commands, steam_client: Res<SteamworksClient>) {
    let client = RenetClient::new(connection_config());

    steam_client.networking_utils().init_relay_network_access();

    println!("From Client {}", steam_client.friends().name());

    let server_steam_id = 76561198079103566;
    let server_steam_id = SteamId::from_raw(server_steam_id);

    let transport = SteamClientTransport::new(&steam_client, &server_steam_id);
    let transport = match transport {
        Ok(transport) => transport,
        Err(e) => {
            println!("Id {:?}", server_steam_id);
            panic!("Error when trying to create SteamClientTransport: {}", e)
        }
    };

    commands.insert_resource(transport);
    commands.insert_resource(client);
    commands.insert_resource(CurrentClientId(steam_client.user().steam_id().raw()));
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
