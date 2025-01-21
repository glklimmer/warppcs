use bevy::{asset::RenderAssetUsages, math::bounding::IntersectsVolume, prelude::*};

use bevy_parallax::ParallaxPlugin;
use bevy_renet::client_connected;
use gizmos::GizmosPlugin;
use image::{GenericImage, GenericImageView, Rgba};
use menu::{MainMenuStates, MenuPlugin};
use networking::{ClientNetworkPlugin, Connected, ControlledPlayer};
use shared::{
    networking::MultiplayerRoles, server::networking::ServerNetworkPlugin, BoxCollider, GameState,
};
use std::f32::consts::PI;
use ui::UiPlugin;

#[cfg(feature = "steam")]
use menu::JoinSteamLobby;

use animations::{objects::flag::GenerateOutline, AnimationPlugin};
use camera::CameraPlugin;
use entities::EntitiesPlugin;
use input::InputPlugin;

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
    let primary_window = Window {
        title: "WARPPCS".to_string(),
        resolution: (1280.0, 720.0).into(),
        resizable: false,
        ..default()
    };
    let mut app = App::new();
    #[cfg(feature = "steam")]
    {
        use shared::steamworks::SteamworksPlugin;
        app.add_plugins(SteamworksPlugin::init_app(1513980).unwrap());
    }

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(primary_window),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    );

    app.insert_state(GameState::MainMenu);
    app.insert_state(MultiplayerRoles::NotInGame);
    app.insert_state(MainMenuStates::TitleScreen);

    app.add_plugins(ParallaxPlugin);
    app.add_plugins(CameraPlugin);
    app.add_plugins(InputPlugin);
    app.add_plugins(AnimationPlugin);
    app.add_plugins(MenuPlugin);
    app.add_plugins(EntitiesPlugin);
    app.add_plugins(UiPlugin);

    app.add_plugins(GizmosPlugin);

    app.add_systems(Startup, setup_background);

    app.add_plugins(ServerNetworkPlugin);
    app.add_plugins(ClientNetworkPlugin);

    #[cfg(feature = "steam")]
    {
        use bevy_renet::steam::{SteamClientPlugin, SteamServerPlugin, SteamTransportError};
        use networking::join_server::{join_own_steam_server, join_steam_server};
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

        app.add_systems(Update, join_steam_server.run_if(on_event::<JoinSteamLobby>));
    }

    #[cfg(feature = "netcode")]
    {
        use bevy_renet::netcode::{
            NetcodeClientPlugin, NetcodeServerPlugin, NetcodeTransportError,
        };

        use networking::join_server::join_netcode_server;
        use shared::server::create_server::create_netcode_server;
        use std::env;

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

        let args: Vec<String> = env::args().collect();

        if args.contains(&String::from("server")) {
            println!("Starting server");
            app.add_systems(
                Startup,
                (create_netcode_server, join_netcode_server).chain(),
            );
        } else {
            println!("Joining server");
            app.add_systems(Startup, join_netcode_server);
        }
    }
    app.add_systems(
        PostUpdate,
        (generate_and_save_outline, check_highlight).run_if(in_state(GameState::GameSession)),
    );
    app.run();
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
#[derive(Component)]
struct ToBe {
    original_handle: Handle<Image>,
}

fn generate_and_save_outline(
    mut sprites: Query<(&mut Sprite), With<ToBe>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (mut texture_handle) in sprites.iter_mut() {
        if let Some(texture) = images.get_mut(texture_handle.image.id()) {
            let width = texture.width() as u32;
            let height = texture.height() as u32;
            let image = texture.clone().try_into_dynamic().unwrap();
            let mut new = image.clone();
            for (x, y, p) in image.pixels() {
                if x == 0 || y == 0 || x == width - 1 || y == height - 1 || p.0[3] != 0 {
                    continue;
                }
                let current = image.get_pixel(x, y)[3];
                let left = image.get_pixel(x - 1, y)[3];
                let right = image.get_pixel(x + 1, y)[3];
                let up = image.get_pixel(x, y - 1)[3];
                let down = image.get_pixel(x, y + 1)[3];
                if current != left || current != right || current != up || current != down {
                    new.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
            }

            let image = Image::from_dynamic(new, true, RenderAssetUsages::RENDER_WORLD);
            texture_handle.image = images.add(image);
        }
    }
}

fn check_highlight(
    mut commands: Commands,
    mut outline: Query<
        (
            Entity,
            &Transform,
            &BoxCollider,
            &mut Sprite,
            Option<&mut ToBe>,
        ),
        (With<GenerateOutline>),
    >,
    player: Query<(&Transform, &BoxCollider), With<ControlledPlayer>>,
) {
    let (player_transform, player_collider) = player.get_single().unwrap();
    let player_bounds = player_collider.at(player_transform);

    for (entity, transform, box_coll, mut sprite, to_be) in outline.iter_mut() {
        let colll = box_coll.at(transform);
        let xx = colll.intersects(&player_bounds);
        match to_be {
            Some(to_be) => {
                if !xx {
                    let original_handle = to_be.original_handle.clone();
                    commands.entity(entity).remove::<ToBe>();
                    sprite.image = original_handle;
                }
            }
            None => {
                if xx {
                    commands.entity(entity).try_insert_if_new(ToBe {
                        original_handle: sprite.image.clone(),
                    });
                }
            }
        }
    }
}
