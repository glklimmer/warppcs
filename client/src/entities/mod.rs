use bevy::prelude::*;

use highlight::HighlightPlugin;
use items::ItemsPlugin;
use spawn::SpawnPlugin;

mod items;
mod spawn;

pub mod highlight;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpawnPlugin)
            .add_plugins(HighlightPlugin)
            .add_plugins(ItemsPlugin);
    }
}
