use bevy::prelude::*;

use shared::{
    enum_map::*,
    map::buildings::{BuildStatus, HealthIndicator},
};

use crate::animations::{
    AnimationSpriteSheet, SpriteSheetAnimation, sprite_variant_loader::SpriteVariants,
};

pub fn wall_wood_building(world: &mut World) -> AnimationSpriteSheet<BuildStatus, SpriteVariants> {
    let asset_server = world.resource::<AssetServer>();
    let texture = asset_server.load("sprites/buildings/wall_2.png");

    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 48),
        15,
        6,
        Some(UVec2::splat(1)),
        None,
    ));

    let animations = EnumMap::new(|c| match c {
        BuildStatus::Marker => SpriteSheetAnimation::default(),
        BuildStatus::Constructing => SpriteSheetAnimation {
            first_sprite_index: 15 * 1 + 0,
            last_sprite_index: 15 * 1 + 14,
            ..default()
        },
        BuildStatus::Built { indicator } => match indicator {
            HealthIndicator::Healthy => SpriteSheetAnimation {
                first_sprite_index: 15 * 0 + 0,
                last_sprite_index: 15 * 0 + 0,
                ..default()
            },
            HealthIndicator::Light => SpriteSheetAnimation {
                first_sprite_index: 15 * 2 + 0,
                last_sprite_index: 15 * 2 + 0,
                ..default()
            },
            HealthIndicator::Medium => SpriteSheetAnimation {
                first_sprite_index: 15 * 3 + 0,
                last_sprite_index: 15 * 3 + 0,
                ..default()
            },
            HealthIndicator::Heavy => SpriteSheetAnimation {
                first_sprite_index: 15 * 4 + 0,
                last_sprite_index: 15 * 4 + 6,
                ..default()
            },
        },
        BuildStatus::Destroyed => SpriteSheetAnimation {
            first_sprite_index: 15 * 5 + 0,
            last_sprite_index: 15 * 5 + 0,
            ..default()
        },
    });

    let animations_sound = EnumMap::new(|c| match c {
        BuildStatus::Marker => None,
        BuildStatus::Constructing => None,
        BuildStatus::Built { indicator: _ } => None,
        BuildStatus::Destroyed => None,
    });

    AnimationSpriteSheet::new(world, texture, layout, animations, animations_sound)
}
