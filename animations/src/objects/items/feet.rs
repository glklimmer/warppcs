use bevy::prelude::*;
use shared::enum_map::*;

use crate::{AnimationSpriteSheet, SpriteSheetAnimation};

#[derive(Debug, Clone, Copy, Mappable)]
pub enum Feet {
    Brown,
    Blue,
    Red,
    Violet,
    Green,
    Beige,
}

#[derive(Resource)]
pub struct FeetSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<Feet, Image>,
}

impl FromWorld for FeetSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture = asset_server.load("sprites/objects/feet_armor.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(25, 25),
            4,
            2,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            Feet::Brown => SpriteSheetAnimation {
                first_sprite_index: 0,
                ..default()
            },
            Feet::Blue => SpriteSheetAnimation {
                first_sprite_index: 1,
                ..default()
            },
            Feet::Red => SpriteSheetAnimation {
                first_sprite_index: 2,
                ..default()
            },
            Feet::Violet => SpriteSheetAnimation {
                first_sprite_index: 3,
                ..default()
            },
            Feet::Green => SpriteSheetAnimation {
                first_sprite_index: 4,
                ..default()
            },
            Feet::Beige => SpriteSheetAnimation {
                first_sprite_index: 5,
                ..default()
            },
        });

        let animations_sound = EnumMap::new(|c| match c {
            Feet::Brown => None,
            Feet::Blue => None,
            Feet::Red => None,
            Feet::Violet => None,
            Feet::Green => None,
            Feet::Beige => None,
        });

        FeetSpriteSheet {
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
