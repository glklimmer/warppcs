use bevy::{prelude::*, sprite::Anchor};

use bevy_replicon::prelude::{AppRuleExt, Replicated};
use health::{DelayedDamage, Health, TakeDamage};
use inventory::Cost;
use items::{MeleeWeapon, ProjectileWeapon, WeaponType};
use lobby::PlayerColor;
use physics::movement::{BoxCollider, RandomVelocityMul, Speed, Velocity};
use serde::{Deserialize, Serialize};
use shared::{enum_map::*, server::entities::UnitAnimation};

use crate::{
    animations::UnitAnimationPlugin,
    death::DeathPlugin,
    pushback::{PushBack, PushbackPlugins},
};

mod animations;
mod death;

pub mod pushback;

pub struct UnitsPlugins;

impl Plugin for UnitsPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((DeathPlugin, UnitAnimationPlugin))
            .replicate_bundle::<(Unit, Transform)>()
            .add_plugins(PushbackPlugins)
            .add_systems(FixedUpdate, unit_swing_timer);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Mappable, PartialEq, Eq)]
pub enum UnitType {
    Shieldwarrior,
    Pikeman,
    Archer,
    Bandit,
    Commander,
}

impl From<WeaponType> for UnitType {
    fn from(value: WeaponType) -> Self {
        match value {
            WeaponType::Melee(use_weapon) => match use_weapon {
                MeleeWeapon::SwordAndShield => UnitType::Shieldwarrior,
                MeleeWeapon::Pike => UnitType::Pikeman,
            },
            WeaponType::Projectile(projectile_weapon) => match projectile_weapon {
                ProjectileWeapon::Bow => UnitType::Archer,
            },
        }
    }
}

impl UnitType {
    pub fn recruitment_cost(&self) -> Cost {
        let gold = match self {
            UnitType::Shieldwarrior => 50,
            UnitType::Pikeman => 50,
            UnitType::Archer => 50,
            UnitType::Bandit => todo!(),
            UnitType::Commander => 100,
        };
        Cost { gold }
    }

    pub fn attack_delayed(&self, damage: TakeDamage) -> DelayedDamage {
        let frame_delay = match self {
            UnitType::Shieldwarrior => 2,
            UnitType::Pikeman => 3,
            UnitType::Archer => 3,
            UnitType::Bandit => 2,
            UnitType::Commander => 2,
        };

        let duration = frame_delay as f32 * 0.1;

        DelayedDamage {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            damage,
        }
    }
}

#[derive(Component, Clone, Deserialize, Serialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = unit_collider(),
    Velocity,
    PushBack,
    UnitAnimation,
    Sprite,
    Anchor::BOTTOM_CENTER,
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

fn unit_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 16.),
        offset: Some(Vec2::new(0., 8.)),
    }
}

fn unit_swing_timer(mut query: Query<&mut Unit, With<Health>>, time: Res<Time>) {
    for mut unit in query.iter_mut() {
        unit.swing_timer.tick(time.delta());
    }
}
