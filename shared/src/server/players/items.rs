use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

use crate::enum_map::*;
use crate::{networking::UnitType, server::physics::movement::Velocity, BoxCollider};

use super::interaction::{Interactable, InteractionType};

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider,
    Velocity,
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
    Interactable(|| Interactable {
        kind: InteractionType::Item,
        restricted_to: None,
    }),
)]
pub struct Item {
    pub item_type: ItemType,
    pub modifiers: Vec<Modifier>,
}

impl Item {
    pub fn random(rarity: Rarity) -> Self {
        let mut item_types = vec![ItemType::Chest, ItemType::Feet, ItemType::Head];

        let weapon = if fastrand::bool() {
            let use_weapon = fastrand::choice(UseWeapon::all_variants()).unwrap();
            WeaponType::Use(*use_weapon)
        } else {
            let proj_weapon = fastrand::choice(ProjectileWeapon::all_variants()).unwrap();
            WeaponType::Projectile(*proj_weapon)
        };
        item_types.push(ItemType::Weapon(weapon));

        let item_type = fastrand::choice(&item_types).unwrap();
        let mut modifiers = item_type.base();

        let multipliers = match rarity {
            Rarity::Common => vec![
                item_type.multiplier(ModifierSign::Positive),
                item_type.multiplier(ModifierSign::Positive),
                item_type.multiplier(ModifierSign::Negative),
            ],
            Rarity::Uncommon => vec![
                item_type.multiplier(ModifierSign::Positive),
                item_type.multiplier(ModifierSign::Negative),
            ],
        };

        modifiers.extend(multipliers);

        Self {
            item_type: *item_type,
            modifiers,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Modifier {
    effect: ModifierEffect,
    modifier_type: ModifierType,
}

#[derive(Clone, Serialize, Deserialize)]
enum ModifierType {
    Amount(i32),
    Multiplier(f32),
}

#[derive(Clone, Serialize, Deserialize, Copy)]
enum ModifierEffect {
    Damage,
    Health,
    Range(WeaponType),
    AttackSpeed,
    MovementSpeed,
    UnitAmount,
}

impl ModifierEffect {
    fn base(&self) -> RangeInclusive<i32> {
        match self {
            ModifierEffect::Damage => 6..=18,
            ModifierEffect::Health => 60..=120,
            ModifierEffect::Range(weapon) => match weapon {
                WeaponType::Use(_) => 20..=30,
                WeaponType::Projectile(_) => 160..=220,
            },
            ModifierEffect::AttackSpeed => 1..=4,
            ModifierEffect::MovementSpeed => 25..=45,
            ModifierEffect::UnitAmount => 3..=5,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Copy)]
pub enum ItemType {
    Weapon(WeaponType),
    Chest,
    Feet,
    Head,
}

enum ModifierSign {
    Positive,
    Negative,
}

impl ItemType {
    fn base(&self) -> Vec<Modifier> {
        let effects = match self {
            ItemType::Weapon(weapon) => vec![
                ModifierEffect::Damage,
                ModifierEffect::AttackSpeed,
                ModifierEffect::Range(*weapon),
            ],
            ItemType::Chest => vec![ModifierEffect::Health],
            ItemType::Feet => vec![ModifierEffect::MovementSpeed],
            ItemType::Head => vec![ModifierEffect::UnitAmount],
        };

        effects
            .iter()
            .map(|effect| {
                let amount = fastrand::i32(effect.base());
                Modifier {
                    effect: *effect,
                    modifier_type: ModifierType::Amount(amount),
                }
            })
            .collect()
    }

    fn multiplier(&self, sign: ModifierSign) -> Modifier {
        // TODO: add this
        todo!()
    }
}

#[derive(Clone, Serialize, Deserialize, Copy)]
pub enum WeaponType {
    Use(UseWeapon),
    Projectile(ProjectileWeapon),
}

#[derive(Clone, Serialize, Deserialize, Copy, Mappable)]
pub enum UseWeapon {
    SwordAndShield,
    Pike,
}

#[derive(Clone, Serialize, Deserialize, Copy, Mappable)]
pub enum ProjectileWeapon {
    Bow,
}

impl WeaponType {
    fn unit_type(&self) -> UnitType {
        match self {
            WeaponType::Use(use_weapon) => match use_weapon {
                UseWeapon::SwordAndShield => UnitType::Shieldwarrior,
                UseWeapon::Pike => UnitType::Pikeman,
            },
            WeaponType::Projectile(projectile_weapon) => match projectile_weapon {
                ProjectileWeapon::Bow => UnitType::Archer,
            },
        }
    }
}
