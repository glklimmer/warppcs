use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::Replicated;
use commander::CommanderPlugin;
use enum_mappable::Mappable;
use health::{Health, HealthPlugin};
use serde::{Deserialize, Serialize};

use crate::{BoxCollider, PlayerColor, enum_map::EnumIter, networking::UnitType, unit_collider};

use super::physics::{
    PushBack,
    movement::{RandomVelocityMul, Speed, Velocity},
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
    BoxCollider = unit_collider(),
    Velocity,
    PushBack,
    UnitAnimation,
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    RandomVelocityMul,
    Health,
    MeleeRange,
    Speed,
    Damage,
    Sight
)]
pub struct Unit {
    pub unit_type: UnitType,
    pub swing_timer: Timer,
    pub color: PlayerColor,
}

#[derive(Component, Debug, Copy, Clone, Deref, DerefMut)]
pub struct Damage(pub f32);

impl Default for Damage {
    fn default() -> Self {
        Self(10.)
    }
}

#[derive(Component, Debug, Copy, Clone, Deref, DerefMut)]
pub struct MeleeRange(pub f32);

#[derive(Component, Debug, Copy, Clone, Deref, DerefMut)]
pub struct ProjectileRange(pub f32);

impl Default for MeleeRange {
    fn default() -> Self {
        Self(10.)
    }
}

#[derive(Component, Debug, Copy, Clone, Deref, DerefMut)]
pub struct Sight(pub f32);

impl Default for Sight {
    fn default() -> Self {
        Self(300.)
    }
}

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HealthPlugin);
        app.add_plugins(CommanderPlugin);

        app.add_systems(FixedUpdate, unit_swing_timer);
    }
}

fn unit_swing_timer(mut query: Query<&mut Unit, With<Health>>, time: Res<Time>) {
    for mut unit in query.iter_mut() {
        unit.swing_timer.tick(time.delta());
    }
}
