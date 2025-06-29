use bevy::prelude::*;
use bevy_parallax::{
    CreateParallaxEvent, LayerData, LayerRepeat, LayerSpeed, ParallaxCameraComponent,
    RepeatStrategy,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands, mut create_parallax: EventWriter<CreateParallaxEvent>) {
    let camera = commands
        .spawn((
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0 / 3.0,
                ..OrthographicProjection::default_2d()
            }),
        ))
        .insert(ParallaxCameraComponent::default())
        .id();
    // TODO: Fix Parralax when travelling
    let event = CreateParallaxEvent {
        layers_data: vec![
            LayerData {
                speed: LayerSpeed::Horizontal(0.9),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/sky.png".to_string(),
                tile_size: UVec2::new(320, 240),
                position: Vec2::new(0., 80.),
                z: -5.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.6),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/far-clouds.png".to_string(),
                tile_size: UVec2::new(128, 240),
                position: Vec2::new(0., 80.),
                z: -4.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.55),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/near-clouds.png".to_string(),
                tile_size: UVec2::new(144, 240),
                position: Vec2::new(0., 80.),
                z: -3.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.45),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/far-mountains.png".to_string(),
                tile_size: UVec2::new(160, 240),
                position: Vec2::new(0., 80.),
                z: -3.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.40),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/mountains.png".to_string(),
                tile_size: UVec2::new(320, 240),
                position: Vec2::new(0., 80.),
                z: -2.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.1),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/trees.png".to_string(),
                tile_size: UVec2::new(240, 240),
                position: Vec2::new(0., 80.),
                z: -1.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/ground.png".to_string(),
                tile_size: UVec2::new(640, 360),
                position: Vec2::new(0., 129.),
                z: -1.0,
                ..default()
            },
        ],
        camera,
    };
    create_parallax.write(event);
}
