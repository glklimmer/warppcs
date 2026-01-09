use bevy::prelude::*;

use crate::props::trees::pine::PineTreeSpriteSheet;

pub(crate) mod trees;

pub(crate) struct PropsAnimationPlugin;

impl Plugin for PropsAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PineTreeSpriteSheet>();
    }
}
