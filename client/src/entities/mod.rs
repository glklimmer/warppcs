use bevy::prelude::*;

use commander::CommanderInteractionPlugin;
use highlight::HighlightPlugin;
use item_assignment::ItemAssignmentPlugin;
use items::ItemsPlugin;
use spawn::SpawnPlugin;

mod item_assignment;
mod spawn;

pub mod commander;
pub mod items;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpawnPlugin)
            .add_plugins(HighlightPlugin)
            .add_plugins(ItemsPlugin)
            .add_plugins(ItemAssignmentPlugin)
            .add_plugins(CommanderInteractionPlugin);
    }
}
