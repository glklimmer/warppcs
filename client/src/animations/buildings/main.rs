use bevy::prelude::*;

use shared::{enum_map::*, map::buildings::BuildStatus};

use crate::animations::{
    AnimationSpriteSheet, SpriteSheetAnimation, sprite_variant_loader::SpriteVariants,
};

pub fn tent(world: &mut World) -> AnimationSpriteSheet<BuildStatus, SpriteVariants> {
    let asset_server = world.resource::<AssetServer>();
    let texture = asset_server.load("sprites/buildings/main_house_blue.png");

    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(64, 48),
        1,
        1,
        None,
        None,
    ));

    let animations = EnumMap::new(|c| match c {
        BuildStatus::Marker => SpriteSheetAnimation {
            first_sprite_index: 0,
            ..default()
        },
        BuildStatus::Built => SpriteSheetAnimation {
            first_sprite_index: 0,
            ..default()
        },
        BuildStatus::Destroyed => SpriteSheetAnimation {
            first_sprite_index: 0,
            ..default()
        },
    });

    let animations_sound = EnumMap::new(|c| match c {
        BuildStatus::Marker => None,
        BuildStatus::Built => None,
        BuildStatus::Destroyed => None,
    });

    AnimationSpriteSheet {
        texture,
        layout,
        animations,
        animations_sound,
    }
}
