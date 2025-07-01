use bevy::prelude::*;
use shared::enum_map::*;

use crate::animations::AnimationSpriteSheet;
use crate::animations::SpriteSheetAnimation;

#[derive(Debug, Clone, Copy, Mappable)]
pub enum Heads {
    Brown,
    Blue,
    Red,
    Violet,
    Green,
    Beige,
}

#[derive(Resource)]
pub struct HeadsSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<Heads, Image>,
}

impl FromWorld for HeadsSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture = asset_server.load("sprites/objects/head_armor.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(25, 25),
            4,
            2,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            Heads::Brown => SpriteSheetAnimation {
                first_sprite_index: 0,
                ..default()
            },
            Heads::Blue => SpriteSheetAnimation {
                first_sprite_index: 1,
                ..default()
            },
            Heads::Red => SpriteSheetAnimation {
                first_sprite_index: 2,
                ..default()
            },
            Heads::Violet => SpriteSheetAnimation {
                first_sprite_index: 3,
                ..default()
            },
            Heads::Green => SpriteSheetAnimation {
                first_sprite_index: 4,
                ..default()
            },
            Heads::Beige => SpriteSheetAnimation {
                first_sprite_index: 5,
                ..default()
            },
        });

        let animations_sound = EnumMap::new(|c| match c {
            Heads::Brown => None,
            Heads::Blue => None,
            Heads::Red => None,
            Heads::Violet => None,
            Heads::Green => None,
            Heads::Beige => None,
        });

        HeadsSpriteSheet {
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
