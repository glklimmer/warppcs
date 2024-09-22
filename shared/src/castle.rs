use bevy::prelude::*;

use super::BoxCollider;

#[derive(Component)]
pub struct TriggerZone;

#[derive(Component)]
pub struct Upgradable;

#[derive(Component)]
pub struct Base;

pub struct CastlePlugin;

impl Plugin for CastlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_castle);
    }
}

fn setup_castle(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Base
    commands.spawn((
        // MaterialMesh2dBundle {
        //     mesh: Mesh2dHandle(meshes.add(Rectangle::new(200.0, 100.0))),
        //     material: materials.add(Color::srgb(255., 255., 255.)),
        //     transform: Transform::from_xyz(0.0, 50.0, 0.0),
        //     ..default()
        // },
        Base,
        BoxCollider(Vec2::new(200., 100.)),
        Upgradable,
        TriggerZone,
    ));
}
