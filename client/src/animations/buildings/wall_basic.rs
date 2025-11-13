use bevy::prelude::*;

use shared::{
    enum_map::*,
    map::buildings::{BuildStatus, HealthIndicator},
};
use sprite_variant_loader::loader::SpriteVariants;

use crate::anim;
use crate::animations::AnimationSpriteSheet;

const ATLAS_COLUMNS: usize = 7;

pub fn wall_basic_building(world: &mut World) -> AnimationSpriteSheet<BuildStatus, SpriteVariants> {
    let asset_server = world.resource::<AssetServer>();
    let texture = asset_server.load("sprites/buildings/wall_1.png");

    let mut layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 16),
        ATLAS_COLUMNS as u32,
        7,
        Some(UVec2::splat(1)),
        None,
    ));

    let animations = EnumMap::new(|status| match status {
        BuildStatus::Marker => anim!(0, 0),
        BuildStatus::Constructing => anim!(2, 3),
        BuildStatus::Built { indicator } => match indicator {
            HealthIndicator::Healthy => anim!(1, 0),
            HealthIndicator::Light => anim!(3, 5),
            HealthIndicator::Medium => anim!(4, 4),
            HealthIndicator::Heavy => anim!(5, 5),
        },
        BuildStatus::Destroyed => anim!(6, 6),
    });

    let animations_sound = EnumMap::new(|_| None);

    AnimationSpriteSheet::new(world, texture, layout, animations, animations_sound)
}
