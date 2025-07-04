use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::movement::{Moving, Velocity};

#[derive(Component, Serialize, Deserialize)]
pub struct AttachedTo(#[entities] pub Entity);

pub struct AttachmentPlugin;

impl Plugin for AttachmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, attachment_follow);
    }
}

const BASE_OFFSET: f32 = 5.0;

fn attachment_follow(
    mut query: Query<(&AttachedTo, &mut Transform)>,
    target: Query<(&GlobalTransform, &Velocity, Option<&Moving>), Without<AttachedTo>>,
    time: Res<Time>,
) {
    for (attached, mut transform) in query.iter_mut() {
        if let Ok((target_transform, velocity, moving)) = target.get(attached.0) {
            let sin_offset = if moving.is_some() {
                (time.elapsed_secs() * 10.0).sin() * 0.75
            } else {
                0.0
            };

            transform.translation.x = target_transform.translation().x;
            transform.translation.y = target_transform.translation().y + BASE_OFFSET + sin_offset;

            let signum = velocity.0.x.signum();
            if signum != 0. {
                transform.scale.x = transform.scale.x.abs() * signum;
            }
        }
    }
}
