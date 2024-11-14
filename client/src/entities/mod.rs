use bevy::prelude::*;

use despawn::DespawnPlugin;
use map::MapPlugin;
use player::PlayerPlugin;
use spawn::SpawnPlugin;

mod despawn;
mod map;
mod player;
mod spawn;

#[derive(Component)]
struct PartOfScene;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpawnPlugin);
        app.add_plugins(PlayerPlugin);
        app.add_plugins(DespawnPlugin);
        app.add_plugins(MapPlugin);
    }
}
