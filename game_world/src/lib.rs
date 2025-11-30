use bevy::prelude::*;

use bevy_replicon::prelude::AppRuleExt;
use init_world::StartGamePlugin;
use shared::GameScene;
use world::WorldPlugin;

pub mod init_world;
pub mod world;

pub struct GameScenesPlugin;

impl Plugin for GameScenesPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<GameScene>()
            .add_plugins((StartGamePlugin, WorldPlugin));
    }
}
