use bevy::prelude::*;

use shared::{
    enum_map::*,
    map::buildings::{BuildStatus, HealthIndicator},
};

use crate::{
    anim,
    animations::{AnimationSpriteSheet, sprite_variant_loader::SpriteVariants},
};

const ATLAS_COLUMNS: usize = 6;

pub fn tent_building(world: &mut World) -> AnimationSpriteSheet<BuildStatus, SpriteVariants> {
    let asset_server = world.resource::<AssetServer>();
    let texture = asset_server.load("sprites/buildings/main_house_blue.png");

    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(64, 48),
        ATLAS_COLUMNS as u32,
        5,
        Some(UVec2::splat(1)),
        None,
    ));

    let animations = EnumMap::new(|c| match c {
        BuildStatus::Marker => anim!(0, 0),
        BuildStatus::Constructing => anim!(2, 10),
        BuildStatus::Built { indicator } => match indicator {
            HealthIndicator::Healthy => anim!(0, 0),
            HealthIndicator::Light => anim!(1, 4),
            HealthIndicator::Medium => anim!(2, 5),
            HealthIndicator::Heavy => anim!(3, 5),
        },
        BuildStatus::Destroyed => anim!(4, 0),
    });

    let animations_sound = EnumMap::new(|c| match c {
        BuildStatus::Marker => None,
        BuildStatus::Constructing => None,
        BuildStatus::Built { indicator: _ } => None,
        BuildStatus::Destroyed => None,
    });

    AnimationSpriteSheet::new(world, texture, layout, animations, animations_sound)
}
