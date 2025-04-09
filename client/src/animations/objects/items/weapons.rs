use bevy::prelude::*;
use shared::enum_map::*;

use crate::animations::SpriteSheet;
use crate::animations::SpriteSheetAnimation;

#[derive(Debug, Clone, Copy, Mappable)]
pub enum Weapons {
    SwordAndShield,
    Pike,
    Bow,
}

#[derive(Resource)]
pub struct WeaponsSpriteSheet {
    pub sprite_sheet: SpriteSheet<Weapons>,
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

        WeaponsSpriteSheet {
            sprite_sheet: SpriteSheet {
                texture,
                layout,
                animations,
            },
        }
    }
}
