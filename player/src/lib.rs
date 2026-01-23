use ai::retreat::General;
use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{AppRuleExt, Replicated};
use health::Health;
use interaction::{InteractionTriggeredEvent, collider_trigger::ColliderTriggerActivater};
use inventory::Inventory;
use lobby::PlayerColor;
use physics::movement::{BoxCollider, Speed, Velocity};
use serde::{Deserialize, Serialize};
use shared::map::Layers;

use crate::{
    animation::PlayerAnimationPlugin, attack::AttackPlugin, client::Client, collect::CollectPlugin,
    commander::CommanderPlugin, defeat::DefeatPlugin, item::pickup_item, knockout::KnockoutPlugin,
    mount::MountPlugin, movement::Movement, teleport::Teleport,
};

mod animation;
mod attack;
mod client;
mod collect;
mod commander;
mod defeat;
mod item;
mod knockout;
mod mount;
mod movement;
mod teleport;

pub struct PlayerPlugins;

impl Plugin for PlayerPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Client,
            AttackPlugin,
            Movement,
            Teleport,
            MountPlugin,
            KnockoutPlugin,
            CommanderPlugin,
            PlayerAnimationPlugin,
            DefeatPlugin,
            CollectPlugin,
        ))
        .replicate_bundle::<(Player, Transform)>()
        .add_systems(
            FixedUpdate,
            pickup_item.run_if(on_message::<InteractionTriggeredEvent>),
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
    Inventory = start_inventory(),
    Health = Health { hitpoints: 200. },
    ColliderTriggerActivater,
    PlayerColor,
    General
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

fn start_inventory() -> Inventory {
    Inventory {
        gold: 600,
        items: Vec::new(),
    }
}
