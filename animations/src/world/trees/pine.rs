use bevy::prelude::*;

use shared::enum_map::*;

use crate::{anim, world::TreeAnimation, AnimationSpriteSheet};

const ATLAS_COLUMNS: usize = 1;

#[derive(Resource)]
pub struct PineTreeSpriteSheet {
    pub bright_sprite_sheet: AnimationSpriteSheet<TreeAnimation, Image>,
    pub dim_sprite_sheet: AnimationSpriteSheet<TreeAnimation, Image>,
    pub dark_sprite_sheet: AnimationSpriteSheet<TreeAnimation, Image>,
}

impl FromWorld for PineTreeSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let bright_texture = asset_server.load("sprites/world/trees/pine/bright.png");
        let dim_texture = asset_server.load("sprites/world/trees/pine/dim.png");
        let dark_texture = asset_server.load("sprites/world/trees/pine/dark.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(34, 52),
            ATLAS_COLUMNS as u32,
            1,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            TreeAnimation::Windy => anim!(0, 0),
        });

        let animations_sound = EnumMap::new(|c| match c {
            TreeAnimation::Windy => None,
        });

        PineTreeSpriteSheet {
            bright_sprite_sheet: AnimationSpriteSheet::new(
                world,
                bright_texture,
                layout.clone(),
                animations.clone(),
                animations_sound.clone(),
            ),
            dim_sprite_sheet: AnimationSpriteSheet::new(
                world,
                dim_texture,
                layout.clone(),
                animations.clone(),
                animations_sound.clone(),
            ),
            dark_sprite_sheet: AnimationSpriteSheet::new(
                world,
                dark_texture,
                layout,
                animations,
                animations_sound,
            ),
        }
    }
}
