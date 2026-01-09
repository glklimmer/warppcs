use bevy::prelude::*;

use crate::chest::ChestPlugin;
use crate::portal::PortalPlugin;

pub mod chest;
pub mod portal;

pub struct InteractablePlugins;

impl Plugin for InteractablePlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((ChestPlugin, PortalPlugin));
    }
}
