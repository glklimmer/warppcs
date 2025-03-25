use bevy::{prelude::*, sprite::Anchor};

use bevy_replicon::prelude::{AppRuleExt, Replicated};
use enum_mappable::Mappable;
use health::HealthPlugin;
use serde::{Deserialize, Serialize};

use crate::{enum_map::EnumIter, networking::UnitType, unit_collider, BoxCollider};

use super::{
    buildings::recruiting::Flag,
    physics::{movement::Velocity, PushBack},
};

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
    BoxCollider(unit_collider),
    Velocity,
    PushBack,
    UnitAnimation,
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
)]
pub struct Unit {
    pub unit_type: UnitType,
    pub swing_timer: Timer,
}

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HealthPlugin);
    }
}
