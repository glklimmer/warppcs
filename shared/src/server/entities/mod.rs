use bevy::{prelude::*, sprite::Anchor};

use bevy_replicon::prelude::Replicated;
use commander::CommanderPlugin;
use enum_mappable::Mappable;
use health::HealthPlugin;
use serde::{Deserialize, Serialize};

use crate::{BoxCollider, enum_map::EnumIter, networking::UnitType, unit_collider};

use super::physics::{
    PushBack,
    movement::{RandomVelocityMul, Velocity},
};
pub mod commander;
pub mod health;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum UnitAnimation {
    #[default]
    Idle,
    Walk,
    Attack,
    Hit,
    Death,
}

#[derive(Component, Clone, Deserialize, Serialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider(unit_collider),
    Velocity,
    PushBack,
    UnitAnimation,
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
    RandomVelocityMul
)]
pub struct Unit {
    pub unit_type: UnitType,
    pub swing_timer: Timer,
}

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HealthPlugin);
        app.add_plugins(CommanderPlugin);
    }
}
