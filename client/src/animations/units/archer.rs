use bevy::prelude::*;

use shared::enum_map::*;

use crate::animations::{SpriteSheet, SpriteSheetAnimation};

use super::UnitAnimation;

pub fn archer(world: &mut World) -> SpriteSheet<UnitAnimation> {
    let asset_server = world.resource::<AssetServer>();
    let texture: Handle<Image> = asset_server.load("sprites/humans/Outline/MiniArcherMan.png");
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
            frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
        },
        UnitAnimation::Walk => SpriteSheetAnimation {
            first_sprite_index: 11,
            last_sprite_index: 16,
            frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
        },
        UnitAnimation::Attack => SpriteSheetAnimation {
            first_sprite_index: 33,
            last_sprite_index: 42,
            frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
        },
        UnitAnimation::Hit => SpriteSheetAnimation {
            first_sprite_index: 55,
            last_sprite_index: 57,
            frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
        },
        UnitAnimation::Death => SpriteSheetAnimation {
            first_sprite_index: 66,
            last_sprite_index: 69,
            frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
        },
    });

    SpriteSheet {
        texture,
        layout,
        animations,
    }
}
