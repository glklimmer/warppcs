use bevy::prelude::*;

use shared::enum_map::*;

use crate::animations::{
    AnimationSound, AnimationSoundTrigger, AnimationSpriteSheet, SpriteSheetAnimation,
    sprite_variant_loader::SpriteVariants,
};

use super::super::UnitAnimation;

pub fn shieldwarrior(world: &mut World) -> AnimationSpriteSheet<UnitAnimation, SpriteVariants> {
    let asset_server = world.resource::<AssetServer>();
    let texture = asset_server.load("sprites/humans/MiniShieldMan.png");

    let attack_sound = asset_server.load("animation_sound/shieldwarrior/sword_hit.ogg");

    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        6,
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
            first_sprite_index: 6,
            last_sprite_index: 11,
            ..default()
        },
        UnitAnimation::Attack => SpriteSheetAnimation {
            first_sprite_index: 18,
            last_sprite_index: 23,
            ..default()
        },
        UnitAnimation::Hit => SpriteSheetAnimation {
            first_sprite_index: 30,
            last_sprite_index: 32,
            ..default()
        },
        UnitAnimation::Death => SpriteSheetAnimation {
            first_sprite_index: 36,
            last_sprite_index: 39,
            ..default()
        },
    });

    let animations_sound = EnumMap::new(|c| match c {
        UnitAnimation::Idle => None,
        UnitAnimation::Walk => None,
        UnitAnimation::Attack => Some(AnimationSound {
            sound_handles: vec![attack_sound.clone()],
            sound_trigger: AnimationSoundTrigger::OnEndFrameTimer,
        }),
        UnitAnimation::Hit => None,
        UnitAnimation::Death => None,
    });

    AnimationSpriteSheet {
        texture,
        layout,
        animations,
        animations_sound,
    }
}
