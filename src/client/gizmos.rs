use bevy::{
    color::palettes::css::{BLUE, GREEN, RED},
    prelude::*,
};

use crate::server::{ai::SIGHT_RANGE, physics::collider::BoxCollider};

pub struct GizmosPlugin;

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_range, draw_collider));
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

fn draw_collider(mut gizmos: Gizmos, query: Query<(&Transform, &BoxCollider)>) {
    for (transform, collider) in query.iter() {
        gizmos.rect_2d(transform.translation.truncate(), 0., collider.0, BLUE);
    }
}
