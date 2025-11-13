use bevy::prelude::*;

use shared::enum_map::*;
use sprite_variant_loader::loader::SpriteVariants;

use crate::{AnimationSound, AnimationSpriteSheet, anim, sound::AnimationSoundTrigger};

use super::super::UnitAnimation;

const ATLAS_COLUMNS: usize = 6;

pub fn shieldwarrior(world: &mut World) -> AnimationSpriteSheet<UnitAnimation, SpriteVariants> {
    let asset_server = world.resource::<AssetServer>();
    let texture = asset_server.load("sprites/humans/MiniShieldMan.png");

    let attack_sound = asset_server.load("animation_sound/shieldwarrior/sword_hit.ogg");

    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        ATLAS_COLUMNS as u32,
        7,
        None,
        None,
    ));

    let animations = EnumMap::new(|c| match c {
        UnitAnimation::Idle => anim!(0, 3),
        UnitAnimation::Walk => anim!(1, 5),
        UnitAnimation::Attack => anim!(3, 5),
        UnitAnimation::Hit => anim!(5, 2),
        UnitAnimation::Death => anim!(6, 3),
    });

    let animations_sound = EnumMap::new(|c| match c {
        UnitAnimation::Idle => None,
        UnitAnimation::Walk => None,
        UnitAnimation::Attack => Some(AnimationSound {
            sound_handles: vec![attack_sound.clone()],
            sound_trigger: AnimationSoundTrigger::EndFrameTimer,
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
