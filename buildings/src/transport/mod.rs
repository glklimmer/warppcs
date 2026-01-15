use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{AppRuleExt, Replicated};
use inventory::Inventory;
use physics::movement::BoxCollider;
use serde::{Deserialize, Serialize};
use shared::{GameSceneId, GameState};

use crate::{BuildStatus, Building};

pub(crate) mod animation;

pub(crate) struct TransportPlugins;

impl Plugin for TransportPlugins {
    fn build(&self, app: &mut App) {
        app.replicate_bundle::<(TransportBuilding, Transform)>()
            .add_systems(
                FixedUpdate,
                (send_transporter.run_if(in_state(GameState::GameSession)),),
            );
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = marker_collider(),
    Sprite,
    Anchor::BOTTOM_CENTER,
    BuildStatus = BuildStatus::Marker,
)]
pub struct TransportBuilding {
    transporters: u8,
}

impl Default for TransportBuilding {
    fn default() -> Self {
        Self { transporters: 1 }
    }
}

fn marker_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(28., 26.),
        offset: Some(Vec2::new(0., 13.)),
    }
}

fn send_transporter(
    transport: Query<(&TransportBuilding, &GameSceneId)>,
    collectables_query: Query<&GameSceneId, (With<Building>, With<Inventory>)>,
) -> Result {
    for game_scene_id in transport.iter() {}
    Ok(())
}
