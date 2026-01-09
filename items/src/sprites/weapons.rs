use bevy::prelude::*;

use crate::{MeleeWeapon, ProjectileWeapon, WeaponType};
use shared::enum_map::*;

use animations::{AnimationSpriteSheet, BuildSprite, SpriteSheetAnimation};

#[derive(Debug, Clone, Copy, Mappable)]
pub enum Weapons {
    SwordAndShield,
    Pike,
    Bow,
    Rapier,
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

#[derive(Resource)]
pub struct WeaponsSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<Weapons, Image>,
}

impl WeaponsSpriteSheet {
    pub fn sprite_for_unit(&self, weapon: Weapons) -> Sprite {
        self.sprite_sheet.sprite_for::<Weapons>(weapon)
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
