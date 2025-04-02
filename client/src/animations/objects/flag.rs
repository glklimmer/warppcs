use bevy::prelude::*;

use shared::enum_map::*;

use crate::animations::{SpriteSheet, SpriteSheetAnimation};

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum FlagAnimation {
    #[default]
    Wave,
}

#[derive(Resource)]
pub struct FlagSpriteSheet {
    pub sprite_sheet: SpriteSheet<FlagAnimation>,
}

impl FromWorld for FlagSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/objects/flag.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(48, 64),
            8,
            1,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            FlagAnimation::Wave => SpriteSheetAnimation {
                first_sprite_index: 0,
                last_sprite_index: 7,
                ..default()
            },
        });

        FlagSpriteSheet {
            sprite_sheet: SpriteSheet {
                texture,
                layout,
                animations,
            },
        }
    }
}
