use bevy::prelude::*;

use shared::PlayerState;

pub mod menu;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(PlayerState::World);
    }
}
