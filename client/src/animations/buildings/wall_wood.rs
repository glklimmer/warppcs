use bevy::prelude::*;

use shared::{
    enum_map::*,
    map::buildings::{BuildStatus, HealthIndicator},
};
use sprite_variant_loader::loader::SpriteVariants;

use crate::anim;
use crate::animations::AnimationSpriteSheet;

const ATLAS_COLUMNS: usize = 15;

pub fn wall_wood_building(world: &mut World) -> AnimationSpriteSheet<BuildStatus, SpriteVariants> {
    let asset_server = world.resource::<AssetServer>();
    let texture = asset_server.load("sprites/buildings/wall_2.png");

    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 48),
        ATLAS_COLUMNS as u32,
        6,
        Some(UVec2::splat(1)),
        None,
    ));

    let animations = EnumMap::new(|c| match c {
        BuildStatus::Marker => anim!(0, 0),
        BuildStatus::Constructing => anim!(1, 14),
        BuildStatus::Built { indicator } => match indicator {
            HealthIndicator::Healthy => anim!(0, 0),
            HealthIndicator::Light => anim!(2, 0),
            HealthIndicator::Medium => anim!(3, 0),
            HealthIndicator::Heavy => anim!(4, 6),
        },
        BuildStatus::Destroyed => anim!(5, 0),
    });

    let animations_sound = EnumMap::new(|c| match c {
        BuildStatus::Marker => None,
        BuildStatus::Constructing => None,
        BuildStatus::Built { indicator: _ } => None,
        BuildStatus::Destroyed => None,
    });

    AnimationSpriteSheet::new(world, texture, layout, animations, animations_sound)
}
