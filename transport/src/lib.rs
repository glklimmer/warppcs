use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{AppRuleExt, Replicated};
use health::Health;
use inventory::Inventory;
use physics::movement::{BoxCollider, Speed, Velocity};
use serde::{Deserialize, Serialize};

use crate::animation::TransporterAnimationPlugin;

mod animation;

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TransporterAnimationPlugin)
            .replicate_bundle::<(Transport, Transform)>();
    }
}

#[derive(Component, Clone, Deserialize, Serialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = transport_collider(),
    Velocity,
    Sprite,
    Anchor::BOTTOM_CENTER,
    Health,
    Speed,
    Inventory
)]
pub struct Transport {
    pub target: Entity,
}

fn transport_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 16.),
        offset: Some(Vec2::new(0., 8.)),
    }
}

#[derive(Component, Debug, Clone)]
pub struct CollectionTarget(pub Entity);

#[derive(Component, Debug, Clone, Deref)]
pub struct HomeBuilding(pub Entity);
