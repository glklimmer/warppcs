use bevy::{prelude::*, sprite::Anchor};

use bevy_replicon::prelude::{AppRuleExt, Replicated};
use health::Health;
use interaction::{InteractionTriggeredEvent, collider_trigger::ColliderTriggerActivater};
use inventory::Inventory;
use lobby::PlayerColor;
use physics::movement::{BoxCollider, Speed, Velocity};
use serde::{Deserialize, Serialize};
use shared::map::Layers;

use crate::{
    attack::Attack,
    chest::{Chest, ChestOpened, open_chest},
    client::Client,
    commander::CommanderPlugin,
    item::pickup_item,
    knockout::KnockoutPlugin,
    mount::MountPlugin,
    movement::Movement,
    teleport::Teleport,
};

pub mod chest;
pub mod knockout;
pub mod mount;

mod attack;
mod client;
mod commander;
mod defeat;
mod item;
mod movement;
mod teleport;

pub struct PlayerPlugins;

impl Plugin for PlayerPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Client,
            Attack,
            Movement,
            Teleport,
            MountPlugin,
            KnockoutPlugin,
            CommanderPlugin,
        ))
        .replicate::<ChestOpened>()
        .replicate_bundle::<(Player, Transform, Inventory)>()
        .replicate_bundle::<(Chest, Transform)>()
        .replicate_bundle::<(Item, Transform)>()
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
    Health = Health { hitpoints: 200. },
    ColliderTriggerActivater,
    PlayerColor
)]
pub struct Player {
    pub id: u64,
}

fn player_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 16.),
        offset: Some(Vec2::new(0., 8.)),
    }
}
