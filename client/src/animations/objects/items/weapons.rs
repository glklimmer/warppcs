use bevy::prelude::*;
use shared::enum_map::*;
use shared::networking::UnitType;

use crate::animations::AnimationSpriteSheet;
use crate::animations::SpriteSheetAnimation;
use crate::entities::items::BuildSprite;

#[derive(Debug, Clone, Copy, Mappable)]
pub enum Weapons {
    SwordAndShield,
    Pike,
    Bow,
}

impl From<UnitType> for Weapons {
    fn from(unit_type: UnitType) -> Self {
        match unit_type {
            UnitType::Shieldwarrior => Weapons::SwordAndShield,
            UnitType::Pikeman => Weapons::Pike,
            UnitType::Archer => Weapons::Bow,
            UnitType::Bandit => todo!(),
            UnitType::Commander => todo!(),
        }
    }
}

#[derive(Resource)]
pub struct WeaponsSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<Weapons>,
}

impl WeaponsSpriteSheet {
    pub fn sprite_for_unit(&self, unit: UnitType) -> Sprite {
        self.sprite_sheet.sprite_for::<Weapons>(unit.into())
    }
}

impl FromWorld for WeaponsSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/objects/weapons.png");
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
        });

        let animations_sound = EnumMap::new(|c| match c {
            Weapons::SwordAndShield => None,
            Weapons::Pike => None,
            Weapons::Bow => None,
        });

        WeaponsSpriteSheet {
            sprite_sheet: AnimationSpriteSheet {
                texture,
                layout,
                animations,
                animations_sound,
            },
        }
    }
}
