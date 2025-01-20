use bevy::prelude::*;
use bevy_parallax::{
    CameraFollow, CreateParallaxEvent, LayerData, LayerRepeat, LayerSpeed, ParallaxCameraComponent,
    RepeatStrategy,
};
use shared::networking::SpawnPlayer;

use super::networking::ControlledPlayer;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, camera_follow_player.run_if(on_event::<SpawnPlayer>));
    }
}

fn setup_camera(mut commands: Commands, mut create_parallax: EventWriter<CreateParallaxEvent>) {
    let camera = commands
        .spawn(Camera2d::default())
        .insert(ParallaxCameraComponent::default())
        .id();
    let event = CreateParallaxEvent {
        layers_data: vec![
            LayerData {
                speed: LayerSpeed::Horizontal(0.9),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/sky.png".to_string(),
                tile_size: UVec2::new(385, 216),
                scale: Vec2::splat(2.5),
                position: Vec2::new(0., 200.),
                z: -3.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.6),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/far_mountains.png".to_string(),
                tile_size: UVec2::new(385, 216),
                scale: Vec2::splat(2.5),
                position: Vec2::new(0., 100.),
                z: -2.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.3),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/grassy_mountains.png".to_string(),
                tile_size: UVec2::new(386, 216),
                scale: Vec2::splat(2.5),
                position: Vec2::new(0., 100.),
                z: -1.0,
                ..default()
            },
        ],
        camera,
    };
    create_parallax.send(event);
}

fn camera_follow_player(
    mut commands: Commands,
    camera: Query<Entity, With<Camera>>,
    player_query: Query<Entity, With<ControlledPlayer>>,
) {
    commands
        .entity(camera.single())
        .insert(CameraFollow::fixed(player_query.single()));
}
