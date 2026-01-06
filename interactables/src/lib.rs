use bevy::prelude::*;

use crate::chest::ChestPlugin;

pub mod chest;

pub struct InteractablePlugins;

impl Plugin for InteractablePlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChestPlugin);
    }
}
