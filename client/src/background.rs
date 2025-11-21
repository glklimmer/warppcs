use bevy::prelude::*;
use bevy_parallax::{CreateParallaxEvent, LayerData, LayerRepeat, LayerSpeed, RepeatStrategy};
use shared::{
    ControlledPlayer, PlayerState,
    server::game_scenes::{travel::Traveling, world::SceneType},
};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_background);
        app.add_systems(OnEnter(PlayerState::Traveling), change_background);
    }
}

fn setup_background(
    camera: Query<Entity, With<Camera2d>>,
    mut create_parallax: EventWriter<CreateParallaxEvent>,
) -> Result {
    let camera = camera.single()?;
    let event = player_background(camera);

    create_parallax.write(event);

    Ok(())
}

fn change_background(
    query: Query<&Traveling, With<ControlledPlayer>>,
    camera: Query<Entity, With<Camera2d>>,
    mut create_parallax: EventWriter<CreateParallaxEvent>,
) -> Result {
    let travel = query.single()?;
    let camera = camera.single()?;

    let event = match travel.target.scene {
        SceneType::Player { .. } => player_background(camera),
        SceneType::Camp { .. } => bandit_background(camera),
        SceneType::Meadow { .. } => bandit_background(camera),
    };

    create_parallax.write(event);

    Ok(())
}

fn player_background(camera: Entity) -> CreateParallaxEvent {
    CreateParallaxEvent {
        layers_data: vec![
            LayerData {
                speed: LayerSpeed::Horizontal(0.9),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/moon.png".to_string(),
                tile_size: UVec2::new(320, 240),
                position: Vec2::new(0., 80.),
                z: -7.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.6),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/far-clouds.png".to_string(),
                tile_size: UVec2::new(128, 240),
                position: Vec2::new(0., 80.),
                z: -6.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.55),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/near-clouds.png".to_string(),
                tile_size: UVec2::new(144, 240),
                position: Vec2::new(0., 80.),
                z: -5.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.45),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/far-mountains.png".to_string(),
                tile_size: UVec2::new(160, 240),
                position: Vec2::new(0., 80.),
                z: -4.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.40),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/mountains.png".to_string(),
                tile_size: UVec2::new(320, 240),
                position: Vec2::new(0., 80.),
                z: -3.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.1),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/trees.png".to_string(),
                tile_size: UVec2::new(240, 240),
                position: Vec2::new(0., 80.),
                z: -2.0,
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
    }
}

fn bandit_background(camera: Entity) -> CreateParallaxEvent {
    CreateParallaxEvent {
        layers_data: vec![
            LayerData {
                speed: LayerSpeed::Horizontal(0.9),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/sky.png".to_string(),
                tile_size: UVec2::new(385, 216),
                position: Vec2::new(0., 70.),
                z: -4.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.6),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/far_mountains.png".to_string(),
                tile_size: UVec2::new(385, 216),
                position: Vec2::new(0., 35.),
                z: -3.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.3),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorBoth),
                path: "background/grassy_mountains.png".to_string(),
                tile_size: UVec2::new(386, 216),
                position: Vec2::new(0., 35.),
                z: -2.0,
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
    }
}
