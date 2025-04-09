use bevy::{
    color::palettes::css::{BLUE, GREEN, RED},
    prelude::*,
};

use shared::{BoxCollider, server::ai::SIGHT_RANGE};

pub struct GizmosPlugin;

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GizmosSettings::default());

        app.add_systems(Update, (draw_range, draw_collider));
    }
}

#[derive(Component)]
pub struct UnitRange(pub f32);

#[derive(Resource, Default)]
pub struct GizmosSettings {
    pub on: bool,
}

fn draw_range(
    mut gizmos: Gizmos,
    settings: Res<GizmosSettings>,
    query: Query<(&Transform, &UnitRange)>,
) {
    if !settings.on {
        return;
    }
    for (transform, range) in query.iter() {
        gizmos.circle_2d(transform.translation.truncate(), range.0, RED);
        gizmos.circle_2d(transform.translation.truncate(), SIGHT_RANGE, GREEN);
    }
}

fn draw_collider(
    mut gizmos: Gizmos,
    settings: Res<GizmosSettings>,
    query: Query<(&Transform, &BoxCollider)>,
) {
    if !settings.on {
        return;
    }
    for (transform, collider) in query.iter() {
        gizmos.rect_2d(
            Isometry2d::new(
                transform.translation.truncate() + collider.offset.unwrap_or_default(),
                Rot2::degrees(0.),
            ),
            collider.dimension,
            BLUE,
        );
    }
}
