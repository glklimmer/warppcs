use bevy::prelude::*;

use attachment::AttachmentPlugin;
use bevy_replicon::prelude::AppRuleExt;
use movement::MovementPlugin;

use crate::movement::BoxCollider;

pub mod attachment;
pub mod movement;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<BoxCollider>()
            .add_plugins((MovementPlugin, AttachmentPlugin));
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum WorldDirection {
    #[default]
    Left,
    Right,
}

impl From<f32> for WorldDirection {
    fn from(value: f32) -> Self {
        match value > 0. {
            true => WorldDirection::Right,
            false => WorldDirection::Left,
        }
    }
}

impl From<WorldDirection> for f32 {
    fn from(value: WorldDirection) -> Self {
        match value {
            WorldDirection::Left => -1.,
            WorldDirection::Right => 1.,
        }
    }
}
