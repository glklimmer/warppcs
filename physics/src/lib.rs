use bevy::prelude::*;

use serde::{Deserialize, Serialize};
use std::time::Duration;

use attachment::AttachmentPlugin;
use collider_trigger::ColliderTriggerPlugin;
use movement::{MovementPlugin, Velocity};

pub mod attachment;
pub mod collider;
pub mod collider_trigger;
pub mod movement;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MovementPlugin, AttachmentPlugin, ColliderTriggerPlugin))
    }
}
