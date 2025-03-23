use bevy::prelude::*;

use despawn::DespawnPlugin;
use highlight::HighlightPlugin;
use map::MapPlugin;
use player::PlayerPlugin;
use shared::server::players::interaction::InteractPlugin;
use spawn::SpawnPlugin;

mod despawn;
mod map;
mod player;
mod spawn;

pub mod highlight;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpawnPlugin).add_plugins(InteractPlugin);
        // app.add_plugins(PlayerPlugin);
        // app.add_plugins(DespawnPlugin);
        // app.add_plugins(MapPlugin);
        // app.add_plugins(HighlightPlugin);
    }
}
