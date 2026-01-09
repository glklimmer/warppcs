use bevy::prelude::*;

use animations::{
    AnimationSound, AnimationSpriteSheet, SpriteVariants, anim,
    sound::{AnimationSoundTrigger, DIRT_FOOTSTEPS_SOUND_PATH},
};
use shared::enum_map::*;

use super::super::UnitAnimation;

const ATLAS_COLUMNS: usize = 11;

pub(crate) fn archer(world: &mut World) -> AnimationSpriteSheet<UnitAnimation, SpriteVariants> {
    let asset_server = world.resource::<AssetServer>();
    let texture = asset_server.load("sprites/humans/MiniArcherMan.png");

    let footstep1 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_1.ogg"));
    let footstep2 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_2.ogg"));
    let footstep3 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_3.ogg"));
    let footstep4 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_4.ogg"));
    let footstep5 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_5.ogg"));
    let footstep6 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_6.ogg"));

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
        UnitAnimation::Attack => anim!(3, 9),
        UnitAnimation::Hit => anim!(5, 2),
        UnitAnimation::Death => anim!(6, 3),
    });

    let animations_sound = EnumMap::new(|c| match c {
        UnitAnimation::Idle => None,
        UnitAnimation::Walk => Some(AnimationSound {
            sound_handles: vec![
                footstep1.clone(),
                footstep2.clone(),
                footstep3.clone(),
                footstep4.clone(),
                footstep5.clone(),
                footstep6.clone(),
            ],
            sound_trigger: AnimationSoundTrigger::StartFrameTimer,
        }),
        UnitAnimation::Attack => None,
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
