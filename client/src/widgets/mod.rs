use bevy::prelude::*;

use quickmenu::QuickMenuPlugin;

mod quickmenu;

pub mod menu;
pub mod text_input;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuickMenuPlugin);
    }
}
