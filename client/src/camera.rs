use bevy::prelude::*;

use super::networking::ControlledPlayer;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, follow_player);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn follow_player(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Transform, (With<ControlledPlayer>, Without<Camera>)>,
) {
    let mut camera_transform = camera_query.single_mut();
    if let Ok(player_transform) = player_query.get_single() {
        camera_transform.translation = player_transform.translation
    }
}
