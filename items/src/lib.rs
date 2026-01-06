use bevy::prelude::*;

use bevy_replicon::prelude::{AppRuleExt, Replicated};
use interaction::{Interactable, InteractionType};
use physics::movement::{BoxCollider, Directionless, Velocity};
use serde::{Deserialize, Serialize};
use shared::enum_map::*;
use std::{cmp::Ordering, fmt, ops::MulAssign};

pub struct ItemPlugins;

impl Plugin for ItemPlugins {
    fn build(&self, app: &mut App) {
        app.replicate_bundle::<(Item, Transform)>();
    }
}

#[derive(Component, Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[require(
    Replicated,
    Transform,
    BoxCollider,
    Velocity,
    Sprite,
    Interactable{
        kind: InteractionType::Item,
        restricted_to: None,
    },
    Directionless
)]
pub struct Item {
    pub item_type: ItemType,
    pub rarity: Rarity,
    pub base: Vec<BaseEffect>,
    pub modifiers: Vec<Modifier>,
    pub color: Option<ItemColor>,
}

#[derive(Default)]
pub struct ItemBuilder {
    rarity: Option<Rarity>,
    item_type: Option<ItemType>,
}

impl ItemBuilder {
    pub fn with_rarity(mut self, rarity: Rarity) -> Self {
        self.rarity = Some(rarity);
        self
    }

    pub fn with_type(mut self, item_type: ItemType) -> Self {
        self.item_type = Some(item_type);
        self
    }

    pub fn with_types(mut self, item_types: Vec<ItemType>) -> Self {
        self.item_type = fastrand::choice(&item_types).copied();
        self
    }

    pub fn build(self) -> Item {
        let rarity = self.rarity.unwrap_or_else(Rarity::random);

        let item_type = self.item_type.unwrap_or_else(ItemType::random);

        Item::generate(rarity, item_type)
    }
}

impl Item {
    pub fn builder() -> ItemBuilder {
        ItemBuilder::default()
    }

    pub fn random() -> Item {
        Self::generate(Rarity::random(), ItemType::random())
    }

    fn generate(rarity: Rarity, item_type: ItemType) -> Self {
        let base = item_type.base();
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
        Self {
            item_type,
            rarity,
            base,
            modifiers: multipliers,
            color: item_type.random_color(),
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
            ItemType::Chest => BoxCollider {
                dimension: Vec2::new(19., 11.),
                offset: None,
            },
            ItemType::Feet => BoxCollider {
                dimension: Vec2::new(19., 16.),
                offset: None,
            },
            ItemType::Head => BoxCollider {
                dimension: Vec2::new(13., 12.),
                offset: None,
            },
        }
    }
}

#[derive(Clone, Copy, Mappable)]
enum ModifierAmplitude {
    Low,
    Middle,
    High,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Mappable, Eq, PartialEq)]
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

    fn random() -> Rarity {
        *fastrand::choice(Rarity::all_variants()).unwrap()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct BaseEffect {
    pub effect: Effect,
    pub amount: i32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Modifier {
    pub effect: Effect,
    pub amount: ModifierAmount,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum ModifierAmount {
    Amount(i32),
    Multiplier(Multiplier),
}

impl ModifierAmount {
    pub fn sign(&self) -> ModifierSign {
        match self {
            ModifierAmount::Amount(amount) => match 0.cmp(amount) {
                Ordering::Less | Ordering::Equal => ModifierSign::Positive,
                Ordering::Greater => ModifierSign::Negative,
            },
            ModifierAmount::Multiplier(multiplier) => multiplier.sign,
        }
    }
}

impl fmt::Display for ModifierAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModifierAmount::Amount(amount) => write!(f, "{amount}"),
            ModifierAmount::Multiplier(multiplier) => write!(f, "{multiplier}"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Multiplier {
    percentage_points: i32,
    sign: ModifierSign,
}

impl Multiplier {
    fn new(percentage_points: i32, sign: ModifierSign) -> Self {
        Self {
            percentage_points,
            sign,
        }
    }

    fn factor(&self) -> f32 {
        match self.sign {
            ModifierSign::Positive => 1. + self.percentage_points as f32 / 100.,
            ModifierSign::Negative => 1. - self.percentage_points as f32 / 100.,
        }
    }
}

impl MulAssign<Multiplier> for f32 {
    fn mul_assign(&mut self, rhs: Multiplier) {
        *self *= rhs.factor();
    }
}

impl fmt::Display for Multiplier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.sign {
            ModifierSign::Positive => write!(f, "{}%", self.percentage_points),
            ModifierSign::Negative => write!(f, "-{}%", self.percentage_points),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Copy, Debug, Eq, PartialEq)]
pub enum Effect {
    Damage,
    Health,
    MeleeRange(WeaponType),
    ProjectileRange(WeaponType),
    AttackSpeed,
    MovementSpeed,
    UnitAmount,
    Sight,
}

impl Effect {
    fn base(&self) -> BaseEffect {
        let range = match self {
            Effect::Damage => 6..=18,
            Effect::Health => 60..=120,
            Effect::MeleeRange(weapon) => match weapon {
                WeaponType::Melee(meele) => match meele {
                    MeleeWeapon::SwordAndShield => 20..=30,
                    MeleeWeapon::Pike => 40..=50,
                },
                WeaponType::Projectile(projectile) => match projectile {
                    ProjectileWeapon::Bow => 10..=15,
                },
            },
            Effect::ProjectileRange(weapon) => match weapon {
                WeaponType::Melee(_) => 0..=0,
                WeaponType::Projectile(projectile) => match projectile {
                    ProjectileWeapon::Bow => 240..=280,
                },
            },
            Effect::AttackSpeed => 10..=12,
            Effect::MovementSpeed => 25..=45,
            Effect::UnitAmount => 4..=4,
            Effect::Sight => 290..=310,
        };
        let amount = fastrand::i32(range);
        BaseEffect {
            effect: *self,
            amount,
        }
    }

    fn multiplier(&self, amplitude: ModifierAmplitude, sign: ModifierSign) -> Modifier {
        let (min, max) = match amplitude {
            ModifierAmplitude::Low => (1, 3),
            ModifierAmplitude::Middle => (3, 5),
            ModifierAmplitude::High => (5, 10),
        };

        let step_size = 5;
        let steps = (max - min) / step_size;

        let amount = min + fastrand::i32(0..=steps) * step_size;

        Modifier {
            effect: *self,
            amount: ModifierAmount::Multiplier(Multiplier::new(amount, sign)),
        }
    }
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Effect::Damage => "Damage",
            Effect::Health => "Health",
            Effect::MeleeRange(_) => "MeleeRange",
            Effect::ProjectileRange(_) => "ProjectileRange",
            Effect::AttackSpeed => "AttackSpeed",
            Effect::MovementSpeed => "MovementSpeed",
            Effect::UnitAmount => "UnitAmount",
            Effect::Sight => "SightRange",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Serialize, Deserialize, Copy, Debug, Eq, PartialEq)]
pub enum ItemType {
    Weapon(WeaponType),
    Chest,
    Feet,
    Head,
}

#[derive(Clone, Serialize, Deserialize, Copy, Debug, Mappable, Eq, PartialEq)]
pub enum ItemColor {
    Brown,
    Blue,
    Red,
    Violet,
    Green,
    Beige,
}

impl ItemType {
    fn random_color(&self) -> Option<ItemColor> {
        if let ItemType::Weapon(_) = self {
            return None;
        }
        Some(*fastrand::choice(ItemColor::all_variants()).unwrap())
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum ModifierSign {
    Positive,
    Negative,
}

impl ItemType {
    fn base(&self) -> Vec<BaseEffect> {
        let effects = match self {
            ItemType::Weapon(weapon) => {
                vec![
                    Effect::Damage,
                    Effect::AttackSpeed,
                    Effect::MeleeRange(*weapon),
                    Effect::ProjectileRange(*weapon),
                ]
            }
            ItemType::Chest => vec![Effect::Health],
            ItemType::Feet => vec![Effect::MovementSpeed],
            ItemType::Head => vec![Effect::UnitAmount, Effect::Sight],
        };

        effects.iter().map(|effect| effect.base()).collect()
    }

    fn multiplier(&self, amplitude: ModifierAmplitude, sign: ModifierSign) -> Modifier {
        let mut effects = vec![
            Effect::Damage,
            Effect::AttackSpeed,
            Effect::Health,
            Effect::MovementSpeed,
        ];

        if let ItemType::Weapon(weapon_type) = self {
            effects.push(Effect::MeleeRange(*weapon_type));
        }

        if let ItemType::Head = self {
            effects.push(Effect::Sight);
        }

        let effect = fastrand::choice(effects).unwrap();
        effect.multiplier(amplitude, sign)
    }

    pub fn all_variants() -> Vec<Self> {
        let mut item_types = vec![ItemType::Chest, ItemType::Feet, ItemType::Head];

        let weapon = if fastrand::bool() {
            let use_weapon = fastrand::choice(MeleeWeapon::all_variants()).unwrap();
            WeaponType::Melee(*use_weapon)
        } else {
            let proj_weapon = fastrand::choice(ProjectileWeapon::all_variants()).unwrap();
            WeaponType::Projectile(*proj_weapon)
        };
        item_types.push(Self::Weapon(weapon));
        item_types
    }

    fn random() -> Self {
        fastrand::choice(Self::all_variants()).unwrap()
    }
}

#[derive(Clone, Serialize, Deserialize, Copy, Debug, Eq, PartialEq)]
pub enum WeaponType {
    Melee(MeleeWeapon),
    Projectile(ProjectileWeapon),
}

#[derive(Clone, Serialize, Deserialize, Copy, Mappable, Debug, Eq, PartialEq)]
pub enum MeleeWeapon {
    SwordAndShield,
    Pike,
}

#[derive(Clone, Serialize, Deserialize, Copy, Mappable, Debug, Eq, PartialEq)]
pub enum ProjectileWeapon {
    Bow,
}

pub trait EffectSelector {
    fn select(&self, item: &Item) -> Option<Effect>;
}

impl<F> EffectSelector for F
where
    F: Fn(&Item) -> Option<Effect>,
{
    fn select(&self, item: &Item) -> Option<Effect> {
        (self)(item)
    }
}

impl EffectSelector for Effect {
    fn select(&self, _item: &Item) -> Option<Effect> {
        Some(*self)
    }
}

pub trait CalculatedStats {
    fn calculated<S>(&self, selector: S) -> f32
    where
        S: EffectSelector;
}

impl CalculatedStats for [Item] {
    fn calculated<S>(&self, selector: S) -> f32
    where
        S: EffectSelector,
    {
        let mut base_sum = 0;
        let mut modifier_amount = 0;
        let mut modifier_factor = 1.;

        for item in self {
            let effect = match selector.select(item) {
                Some(e) => e,
                None => continue,
            };

            base_sum += item
                .base
                .iter()
                .filter(|b| b.effect == effect)
                .map(|b| b.amount)
                .sum::<i32>();

            for modifier in &item.modifiers {
                if modifier.effect != effect {
                    continue;
                }
                match &modifier.amount {
                    ModifierAmount::Amount(amount) => modifier_amount += *amount,
                    ModifierAmount::Multiplier(multiplier) => modifier_factor *= multiplier.clone(),
                }
            }
        }

        (base_sum + modifier_amount) as f32 * modifier_factor
    }
}
