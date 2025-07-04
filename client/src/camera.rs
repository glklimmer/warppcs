use bevy::prelude::*;
use bevy_parallax::ParallaxCameraComponent;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn((
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0 / 3.0,
                ..OrthographicProjection::default_2d()
            }),
        ))
        .insert(ParallaxCameraComponent::default());
}
