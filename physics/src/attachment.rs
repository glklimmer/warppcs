use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use super::movement::{Moving, Velocity};

use crate::player_attacks::AttackIndicator;

#[derive(Component, Serialize, Deserialize, Deref)]
pub struct AttachedTo(#[entities] pub Entity);

pub struct AttachmentPlugin;

impl Plugin for AttachmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, attachment_follow)
            .add_observer(on_attachment_removed);
    }
}

const X_OFFSET: f32 = 2.0;
const Y_OFFSET: f32 = 5.0;

fn attachment_follow(
    mut query: Query<(&AttachedTo, &mut Transform, Option<&AttackIndicator>)>,
    target: Query<(&GlobalTransform, &Velocity, Option<&Moving>), Without<AttachedTo>>,
    time: Res<Time>,
) -> Result {
    for (attached, mut transform, attack_indicator) in query.iter_mut() {
        let (target_transform, velocity, moving) = target.get(**attached)?;
        let sin_offset = if moving.is_some() {
            (time.elapsed_secs() * 10.0).sin() * 0.75
        } else {
            0.0
        };

        transform.translation.x = target_transform.translation().x + X_OFFSET;
        transform.translation.y = target_transform.translation().y + Y_OFFSET + sin_offset;

        if let Some(indicator) = attack_indicator {
            let world_direction: f32 = indicator.direction.into();
            let angle = -45.0f32.to_radians() * world_direction;

            transform.rotation = Quat::from_rotation_z(angle);
            transform.scale.x = transform.scale.x.abs() * world_direction;
        } else {
            transform.rotation = Quat::IDENTITY;
            let signum = velocity.0.x.signum();
            if signum != 0. {
                transform.scale.x = transform.scale.x.abs() * signum;
            }
        }
    }
    Ok(())
}

fn on_attachment_removed(
    trigger: On<Remove, AttachedTo>,
    mut query: Query<&mut Transform>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.entity;
    let mut transform = query.get_mut(entity)?;
    transform.rotation = Quat::IDENTITY;
    commands.entity(entity).remove::<AttackIndicator>();
    Ok(())
}
