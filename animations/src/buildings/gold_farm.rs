use bevy::prelude::*;

use buildings::{BuildStatus, HealthIndicator};
use shared::enum_map::*;
use sprite_variant_loader::loader::SpriteVariants;

use crate::AnimationSpriteSheet;
use crate::anim;

const ATLAS_COLUMNS: usize = 7;

pub fn gold_farm_building(world: &mut World) -> AnimationSpriteSheet<BuildStatus, SpriteVariants> {
    let asset_server = world.resource::<AssetServer>();
    let texture = asset_server.load("sprites/buildings/gold_mine.png");

    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(90, 50),
        ATLAS_COLUMNS as u32,
        7,
        Some(UVec2::splat(1)),
        None,
    ));

    let animations = EnumMap::new(|c| match c {
        BuildStatus::Marker => anim!(0, 0),
        BuildStatus::Constructing => anim!(1, 6),
        BuildStatus::Built { indicator } => match indicator {
            HealthIndicator::Healthy => anim!(2, 0),
            HealthIndicator::Light => anim!(3, 1),
            HealthIndicator::Medium => anim!(4, 2),
            HealthIndicator::Heavy => anim!(5, 6),
        },
        BuildStatus::Destroyed => anim!(6, 0),
    });

    let animations_sound = EnumMap::new(|c| match c {
        BuildStatus::Marker => None,
        BuildStatus::Constructing => None,
        BuildStatus::Built { indicator: _ } => None,
        BuildStatus::Destroyed => None,
    });

    AnimationSpriteSheet::new(world, texture, layout, animations, animations_sound)
}
