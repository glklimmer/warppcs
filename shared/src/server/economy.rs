pub struct EconomyPlugin;
use bevy::prelude::*;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, setup_ui);
    }
}
