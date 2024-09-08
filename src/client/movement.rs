use bevy::prelude::*;

use crate::shared::networking::Movement;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, set_transform);
    }
}

fn set_transform(mut query: Query<(&mut Transform, &Movement)>) {
    for (mut transform, movement) in &mut query {
        transform.translation = movement.translation.into();
    }
}
