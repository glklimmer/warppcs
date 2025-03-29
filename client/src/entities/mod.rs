use bevy::prelude::*;

use highlight::HighlightPlugin;
use spawn::SpawnPlugin;

mod spawn;

pub mod highlight;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpawnPlugin);
        app.add_plugins(HighlightPlugin);
    }
}
