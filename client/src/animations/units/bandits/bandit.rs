use bevy::prelude::*;

use shared::enum_map::*;

use crate::{
    animations::{
        AnimationSound, AnimationSoundTrigger, AnimationSpriteSheet, SpriteSheetAnimation,
        colored_sprite_loader::SpriteVariants,
    },
    sound::DIRT_FOOTSTEPS_SOUND_PATH,
};

use super::super::UnitAnimation;

pub fn bandit(world: &mut World) -> AnimationSpriteSheet<UnitAnimation, SpriteVariants> {
    let asset_server = world.resource::<AssetServer>();
    let texture = asset_server.load("sprites/bandits/MiniBandit.png");

    let footstep1 = asset_server.load(format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_1.ogg"));
    let footstep2 = asset_server.load(format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_2.ogg"));
    let footstep3 = asset_server.load(format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_3.ogg"));
    let footstep4 = asset_server.load(format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_4.ogg"));
    let footstep5 = asset_server.load(format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_5.ogg"));
    let footstep6 = asset_server.load(format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_6.ogg"));

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
            last_sprite_index: 22,
            ..default()
        },
        UnitAnimation::Hit => SpriteSheetAnimation {
            first_sprite_index: 30,
            last_sprite_index: 31,
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
        UnitAnimation::Walk => Some(AnimationSound {
            sound_handles: vec![
                footstep1.clone(),
                footstep2.clone(),
                footstep3.clone(),
                footstep4.clone(),
                footstep5.clone(),
                footstep6.clone(),
            ],
            sound_trigger: AnimationSoundTrigger::OnStartFrameTimer,
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
