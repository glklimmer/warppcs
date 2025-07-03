use bevy::prelude::*;
use shared::enum_map::*;

use crate::animations::AnimationSpriteSheet;
use crate::animations::SpriteSheetAnimation;

#[derive(Debug, Clone, Copy, Mappable)]
pub enum Chests {
    Brown,
    Blue,
    Red,
    Violet,
    Green,
    Beige,
}

#[derive(Resource)]
pub struct ChestsSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<Chests, Image>,
}

impl FromWorld for ChestsSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture = asset_server.load("sprites/objects/chest_armor.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(25, 25),
            4,
            2,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            Chests::Brown => SpriteSheetAnimation {
                first_sprite_index: 0,
                ..default()
            },
            Chests::Blue => SpriteSheetAnimation {
                first_sprite_index: 1,
                ..default()
            },
            Chests::Red => SpriteSheetAnimation {
                first_sprite_index: 2,
                ..default()
            },
            Chests::Violet => SpriteSheetAnimation {
                first_sprite_index: 3,
                ..default()
            },
            Chests::Green => SpriteSheetAnimation {
                first_sprite_index: 4,
                ..default()
            },
            Chests::Beige => SpriteSheetAnimation {
                first_sprite_index: 5,
                ..default()
            },
        });

        let animations_sound = EnumMap::new(|c| match c {
            Chests::Brown => None,
            Chests::Blue => None,
            Chests::Red => None,
            Chests::Violet => None,
            Chests::Green => None,
            Chests::Beige => None,
        });

        ChestsSpriteSheet {
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
