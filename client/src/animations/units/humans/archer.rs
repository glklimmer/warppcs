use bevy::prelude::*;

use shared::{enum_map::*, server::entities::UnitAnimation};

use crate::animations::{SpriteSheet, SpriteSheetAnimation};

pub fn archer(world: &mut World) -> SpriteSheet<UnitAnimation> {
    let asset_server = world.resource::<AssetServer>();
    let texture: Handle<Image> = asset_server.load("sprites/humans/MiniArcherMan.png");
    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        11,
        7,
        None,
        None,
    ));

    let animations = EnumMap::new(|c| match c {
        UnitAnimation::Idle => SpriteSheetAnimation {
            first_sprite_index: 0,
            last_sprite_index: 3,
            ..default()
        },
        UnitAnimation::Walk => SpriteSheetAnimation {
            first_sprite_index: 11,
            last_sprite_index: 16,
            ..default()
        },
        UnitAnimation::Attack => SpriteSheetAnimation {
            first_sprite_index: 33,
            last_sprite_index: 42,
            ..default()
        },
        UnitAnimation::Hit => SpriteSheetAnimation {
            first_sprite_index: 55,
            last_sprite_index: 57,
            ..default()
        },
        UnitAnimation::Death => SpriteSheetAnimation {
            first_sprite_index: 66,
            last_sprite_index: 69,
            ..default()
        },
    });

    SpriteSheet {
        texture,
        layout,
        animations,
    }
}
