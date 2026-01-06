use bevy::prelude::*;

use items::{ItemColor, MeleeWeapon, ProjectileWeapon, WeaponType};
use shared::enum_map::*;
use units::UnitType;

use crate::{
    AnimationSpriteSheet, BuildSprite, SpriteSheetAnimation,
    objects::items::{chests::Chests, feet::Feet, heads::Heads},
};

#[derive(Debug, Clone, Copy, Mappable)]
pub enum Weapons {
    SwordAndShield,
    Pike,
    Bow,
    Rapier,
}

impl From<UnitType> for Weapons {
    fn from(unit_type: UnitType) -> Self {
        match unit_type {
            UnitType::Shieldwarrior => Weapons::SwordAndShield,
            UnitType::Pikeman => Weapons::Pike,
            UnitType::Archer => Weapons::Bow,
            UnitType::Bandit => todo!(),
            UnitType::Commander => Weapons::Rapier,
        }
    }
}

impl From<WeaponType> for Weapons {
    fn from(wt: WeaponType) -> Self {
        match wt {
            WeaponType::Melee(m) => match m {
                MeleeWeapon::SwordAndShield => Weapons::SwordAndShield,
                MeleeWeapon::Pike => Weapons::Pike,
            },
            WeaponType::Projectile(p) => match p {
                ProjectileWeapon::Bow => Weapons::Bow,
            },
        }
    }
}

impl From<ItemColor> for Chests {
    fn from(c: ItemColor) -> Self {
        match c {
            ItemColor::Brown => Chests::Brown,
            ItemColor::Blue => Chests::Blue,
            ItemColor::Red => Chests::Red,
            ItemColor::Violet => Chests::Violet,
            ItemColor::Green => Chests::Green,
            ItemColor::Beige => Chests::Beige,
        }
    }
}

impl From<ItemColor> for Heads {
    fn from(c: ItemColor) -> Self {
        match c {
            ItemColor::Brown => Heads::Brown,
            ItemColor::Blue => Heads::Blue,
            ItemColor::Red => Heads::Red,
            ItemColor::Violet => Heads::Violet,
            ItemColor::Green => Heads::Green,
            ItemColor::Beige => Heads::Beige,
        }
    }
}

impl From<ItemColor> for Feet {
    fn from(c: ItemColor) -> Self {
        match c {
            ItemColor::Brown => Feet::Brown,
            ItemColor::Blue => Feet::Blue,
            ItemColor::Red => Feet::Red,
            ItemColor::Violet => Feet::Violet,
            ItemColor::Green => Feet::Green,
            ItemColor::Beige => Feet::Beige,
        }
    }
}

#[derive(Resource)]
pub struct WeaponsSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<Weapons, Image>,
}

impl WeaponsSpriteSheet {
    pub fn sprite_for_unit(&self, unit: UnitType) -> Sprite {
        self.sprite_sheet.sprite_for::<Weapons>(unit.into())
    }
}

impl FromWorld for WeaponsSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture = asset_server.load("sprites/objects/weapons.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(25, 25),
            4,
            3,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            Weapons::SwordAndShield => SpriteSheetAnimation {
                first_sprite_index: 7,
                ..default()
            },
            Weapons::Pike => SpriteSheetAnimation {
                first_sprite_index: 4,
                ..default()
            },
            Weapons::Bow => SpriteSheetAnimation {
                first_sprite_index: 5,
                ..default()
            },
            Weapons::Rapier => SpriteSheetAnimation {
                first_sprite_index: 0,
                ..default()
            },
        });

        let animations_sound = EnumMap::new(|c| match c {
            Weapons::SwordAndShield => None,
            Weapons::Pike => None,
            Weapons::Bow => None,
            Weapons::Rapier => None,
        });

        WeaponsSpriteSheet {
            sprite_sheet: AnimationSpriteSheet::new(
                world,
                texture,
                layout,
                animations,
                animations_sound,
            ),
        }
    }
}
