use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{AppRuleExt, Replicated};
use health::Health;
use inventory::Inventory;
use physics::movement::{BoxCollider, RandomVelocityMul, Speed, Velocity};
use serde::{Deserialize, Serialize};
use shared::GameState;

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
    RandomVelocityMul,
    Health,
    Speed,
)]
pub struct Transport;

fn transport_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 16.),
        offset: Some(Vec2::new(0., 8.)),
    }
}
