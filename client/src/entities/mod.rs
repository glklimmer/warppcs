use bevy::prelude::*;

use despawn::DespawnPlugin;
use highlight::HighlightPlugin;
use map::MapPlugin;
use player::PlayerPlugin;
use spawn::SpawnPlugin;

mod despawn;
pub mod highlight;
mod map;
pub mod player;
mod spawn;

#[derive(Component, Default)]
pub struct PartOfScene;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpawnPlugin);
        app.add_plugins(PlayerPlugin);
        app.add_plugins(DespawnPlugin);
        app.add_plugins(MapPlugin);
        app.add_plugins(HighlightPlugin);
    }
}
