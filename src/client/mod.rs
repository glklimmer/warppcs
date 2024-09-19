use bevy::prelude::*;

use animation::AnimationPlugin;
use camera::CameraPlugin;
use input::InputPlugin;
use king::KingPlugin;
use networking::ClientNetworkingPlugin;

use crate::shared::networking::setup_level;

pub mod animation;
pub mod camera;
pub mod gizmos;
pub mod input;
pub mod king;
pub mod networking;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(KingPlugin);
        app.add_plugins(ClientNetworkingPlugin);
        app.add_plugins(CameraPlugin);
        app.add_plugins(InputPlugin);
        app.add_plugins(AnimationPlugin);

        app.add_systems(Startup, setup_level);
    }
}
