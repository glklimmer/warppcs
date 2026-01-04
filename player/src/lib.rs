use bevy::{prelude::*, sprite::Anchor};

use bevy_replicon::prelude::Replicated;
use inventory::Inventory;
use physics::{
    collider::BoxCollider,
    movement::{Speed, Velocity},
};
use serde::{Deserialize, Serialize};

use interaction::InteractionTriggeredEvent;
use shared::PlayerColor;

use crate::{
    attack::Attack, chest::open_chest, items::pickup_item, knockout::KnockoutPlugin,
    mount::MountPlugin, movement::Movement, teleport::Teleport,
};

pub mod chest;
pub mod items;
pub mod knockout;
pub mod mount;

mod attack;
mod movement;
mod teleport;

pub struct PlayerPlugins;

impl Plugin for PlayerPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((Attack, Movement, Teleport))
            .add_plugins((MountPlugin, KnockoutPlugin))
            .add_systems(
                FixedUpdate,
                (open_chest, pickup_item).run_if(on_message::<InteractionTriggeredEvent>),
            );
    }
}

#[derive(Component, Deserialize, Serialize)]
#[require(
    Replicated,
    Transform = Transform::from_xyz(0., 0., Layers::Player.as_f32()),
    BoxCollider = player_collider(),
    Speed,
    Velocity,
    Sprite,
    Anchor::BOTTOM_CENTER,
    Inventory,
)]
pub struct Player {
    pub id: u64,
    pub color: PlayerColor,
}

fn player_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 16.),
        offset: Some(Vec2::new(0., 8.)),
    }
}
