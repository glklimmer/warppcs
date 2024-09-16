use bevy::{
    color::palettes::css::{GREEN, RED},
    prelude::*,
};

use crate::server::ai::SIGHT_RANGE;

pub struct GizmosPlugin;

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_range);
    }
}

#[derive(Component)]
pub struct UnitRange(pub f32);

fn draw_range(mut gizmos: Gizmos, query: Query<(&Transform, &UnitRange)>) {
    for (transform, range) in query.iter() {
        gizmos.circle_2d(transform.translation.truncate(), range.0, RED);
        gizmos.circle_2d(transform.translation.truncate(), SIGHT_RANGE, GREEN);
    }
}
