use bevy::prelude::*;

use shared::enum_map::*;

use crate::animations::{SpriteSheet, SpriteSheetAnimation};

use super::UnitAnimation;

pub fn archer(world: &mut World) -> SpriteSheet<UnitAnimation> {
    let asset_server = world.resource::<AssetServer>();
    let texture: Handle<Image> = asset_server.load("sprites/humans/Outline/MiniArcherMan.png");
    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let idle = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        0,
        4,
        None,
        None,
    ));

    let walk = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        0,
        6,
        None,
        Some(UVec2::new(0, 32)),
    ));

    let attack = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        0,
        11,
        None,
        Some(UVec2::new(0, 32 * 3)),
    ));

    let animations = EnumMap::new(|c| match c {
        UnitAnimation::Idle => SpriteSheetAnimation {
            layout: idle.clone(),
            first_sprite_index: 0,
            last_sprite_index: 3,
            frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
        },
        UnitAnimation::Walk => SpriteSheetAnimation {
            layout: walk.clone(),
            first_sprite_index: 0,
            last_sprite_index: 5,
            frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
        },
        UnitAnimation::Attack => SpriteSheetAnimation {
            layout: attack.clone(),
            first_sprite_index: 0,
            last_sprite_index: 10,
            frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
        },
    });

    SpriteSheet {
        texture,
        animations,
    }
}
