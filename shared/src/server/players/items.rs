use bevy::prelude::*;

use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{BoxCollider, enum_map::*, networking::UnitType, server::physics::movement::Velocity};

use super::interaction::{Interactable, InteractionType};

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
#[require(
    Replicated,
    Transform,
    BoxCollider,
    Velocity,
    Sprite,
    Interactable(|| Interactable {
        kind: InteractionType::Item,
        restricted_to: None,
    }),
)]
pub struct Item {
    pub item_type: ItemType,
    pub rarity: Rarity,
    pub modifiers: Vec<Modifier>,
}

impl Item {
    pub fn random(rarity: Rarity) -> Self {
        let mut item_types = vec![ItemType::Chest, ItemType::Feet, ItemType::Head];
        let mut item_types = vec![];

        let weapon = if fastrand::bool() {
            let use_weapon = fastrand::choice(MeleeWeapon::all_variants()).unwrap();
            WeaponType::Melee(*use_weapon)
        } else {
            let proj_weapon = fastrand::choice(ProjectileWeapon::all_variants()).unwrap();
            WeaponType::Projectile(*proj_weapon)
        };
        item_types.push(ItemType::Weapon(weapon));

        let item_type = fastrand::choice(&item_types).unwrap();
        let mut modifiers = item_type.base();

        let amplitude = *fastrand::choice(ModifierAmplitude::all_variants()).unwrap();
        let multipliers = match rarity {
            Rarity::Common => vec![
                item_type.multiplier(amplitude, ModifierSign::Positive),
                item_type.multiplier(amplitude, ModifierSign::Negative),
                item_type.multiplier(amplitude, ModifierSign::Negative),
            ],
            Rarity::Uncommon => vec![
                item_type.multiplier(amplitude, ModifierSign::Positive),
                item_type.multiplier(amplitude, ModifierSign::Negative),
            ],
        };

        modifiers.extend(multipliers);

        Self {
            item_type: *item_type,
            rarity,
            modifiers,
        }
    }

    pub fn collider(&self) -> BoxCollider {
        match self.item_type {
            ItemType::Weapon(weapon_type) => match weapon_type {
                WeaponType::Melee(melee_weapon) => match melee_weapon {
                    MeleeWeapon::SwordAndShield => BoxCollider {
                        dimension: Vec2::new(12., 14.),
                        offset: None,
                    },
                    MeleeWeapon::Pike => BoxCollider {
                        dimension: Vec2::new(5., 15.),
                        offset: None,
                    },
                },
                WeaponType::Projectile(projectile_weapon) => match projectile_weapon {
                    ProjectileWeapon::Bow => BoxCollider {
                        dimension: Vec2::new(5., 12.),
                        offset: None,
                    },
                },
            },
            ItemType::Chest => todo!(),
            ItemType::Feet => todo!(),
            ItemType::Head => todo!(),
        }
    }
}

#[derive(Clone, Copy, Mappable)]
enum ModifierAmplitude {
    Low,
    Middle,
    High,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Rarity {
    Common,
    Uncommon,
}

impl Rarity {
    pub fn color(&self) -> Color {
        match self {
            Rarity::Common => Color::srgb(0.62, 0.62, 0.62),
            Rarity::Uncommon => Color::srgb(0.12, 1.0, 0.0),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Modifier {
    pub effect: ModifierEffect,
    pub amount: ModifierAmount,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ModifierAmount {
    Base(i32),
    Multiplier(i32),
}

impl fmt::Display for ModifierAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModifierAmount::Base(amount) => write!(f, "{}", amount),
            ModifierAmount::Multiplier(multiplier) => write!(f, "{}%", multiplier),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Copy, Debug)]
pub enum ModifierEffect {
    Damage,
    Health,
    Range(WeaponType),
    AttackSpeed,
    MovementSpeed,
    UnitAmount,
}

impl ModifierEffect {
    fn base(&self) -> Modifier {
        let range = match self {
            ModifierEffect::Damage => 6..=18,
            ModifierEffect::Health => 60..=120,
            ModifierEffect::Range(weapon) => match weapon {
                WeaponType::Melee(_) => 20..=30,
                WeaponType::Projectile(_) => 160..=220,
            },
            ModifierEffect::AttackSpeed => 1..=4,
            ModifierEffect::MovementSpeed => 25..=45,
            ModifierEffect::UnitAmount => 3..=5,
        };
        let amount = fastrand::i32(range);
        Modifier {
            effect: *self,
            amount: ModifierAmount::Base(amount),
        }
    }

    fn multiplier(&self, amplitude: ModifierAmplitude, sign: ModifierSign) -> Modifier {
        let (min, max) = match amplitude {
            ModifierAmplitude::Low => (5, 20),
            ModifierAmplitude::Middle => (20, 50),
            ModifierAmplitude::High => (50, 100),
        };

        let step_size = 5;
        let steps = (max - min) / step_size;

        let mut amount = min + fastrand::i32(0..=steps) * step_size;

        if let ModifierSign::Negative = sign {
            amount = -amount;
        }

        Modifier {
            effect: *self,
            amount: ModifierAmount::Multiplier(amount),
        }
    }
}

impl fmt::Display for ModifierEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ModifierEffect::Damage => "Damage",
            ModifierEffect::Health => "Health",
            ModifierEffect::Range(_) => "Range",
            ModifierEffect::AttackSpeed => "AttackSpeed",
            ModifierEffect::MovementSpeed => "MovementSpeed",
            ModifierEffect::UnitAmount => "UnitAmount",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Serialize, Deserialize, Copy, Debug)]
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

        effects.iter().map(|effect| effect.base()).collect()
    }

    fn multiplier(&self, amplitude: ModifierAmplitude, sign: ModifierSign) -> Modifier {
        let mut effects = vec![
            ModifierEffect::Damage,
            ModifierEffect::AttackSpeed,
            ModifierEffect::Health,
            ModifierEffect::MovementSpeed,
        ];

        if let ItemType::Weapon(weapon_type) = self {
            effects.push(ModifierEffect::Range(*weapon_type));
        }

        let effect = fastrand::choice(effects).unwrap();
        effect.multiplier(amplitude, sign)
    }
}

#[derive(Clone, Serialize, Deserialize, Copy, Debug)]
pub enum WeaponType {
    Melee(MeleeWeapon),
    Projectile(ProjectileWeapon),
}

#[derive(Clone, Serialize, Deserialize, Copy, Mappable, Debug)]
pub enum MeleeWeapon {
    SwordAndShield,
    Pike,
}

#[derive(Clone, Serialize, Deserialize, Copy, Mappable, Debug)]
pub enum ProjectileWeapon {
    Bow,
}

impl WeaponType {
    fn unit_type(&self) -> UnitType {
        match self {
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
