use std::f32::consts::PI;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use animation::AnimationPlugin;
use camera::CameraPlugin;
use input::InputPlugin;
use king::KingPlugin;
use networking::ClientNetworkingPlugin;
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
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));

    app.add_plugins(KingPlugin);
    app.add_plugins(ClientNetworkingPlugin);
    app.add_plugins(CameraPlugin);
    app.add_plugins(InputPlugin);
    app.add_plugins(AnimationPlugin);
    app.add_plugins(MenuPlugin);

    app.add_systems(Startup, setup_background);

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
