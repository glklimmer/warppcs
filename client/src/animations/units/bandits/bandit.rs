use bevy::prelude::*;

use shared::enum_map::*;

use crate::{
    animations::{AnimationSound, AnimationSoundTrigger, SpriteSheet, SpriteSheetAnimation},
    sound::GRASS_FOOTSTEPS_SOUND_PATH,
};

use super::super::UnitAnimation;

pub fn bandit(world: &mut World) -> SpriteSheet<UnitAnimation> {
    let asset_server = world.resource::<AssetServer>();
    let texture: Handle<Image> = asset_server.load("sprites/bandits/MiniBandit.png");
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
        UnitAnimation::Idle => AnimationSound {
            sound_files: vec![],
            sound_trigger: AnimationSoundTrigger::OnEnter,
        },
        UnitAnimation::Walk => AnimationSound {
            sound_files: vec![
                format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_1.ogg"),
                format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_2.ogg"),
                format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_3.ogg"),
                format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_4.ogg"),
                format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_5.ogg"),
                format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_6.ogg"),
            ],
            sound_trigger: AnimationSoundTrigger::OnStartFrameTimer,
        },
        UnitAnimation::Attack => AnimationSound {
            sound_files: vec![],
            sound_trigger: AnimationSoundTrigger::OnEnter,
        },
        UnitAnimation::Hit => AnimationSound {
            sound_files: vec![],
            sound_trigger: AnimationSoundTrigger::OnEnter,
        },
        UnitAnimation::Death => AnimationSound {
            sound_files: vec![],
            sound_trigger: AnimationSoundTrigger::OnEnter,
        },
    });

    SpriteSheet {
        texture,
        layout,
        animations,
        animations_sound,
    }
}
