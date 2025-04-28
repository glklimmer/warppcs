use bevy::prelude::*;

use quickmenu::QuickMenuPlugin;
use shared::PlayerState;

mod quickmenu;

pub mod menu;
pub mod text_input;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuickMenuPlugin);
        app.insert_state(PlayerState::World);
    }
}
