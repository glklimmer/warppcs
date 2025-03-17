use bevy::prelude::*;

use shared::enum_map::*;

use crate::{
    animations::{AnimationSound, AnimationSoundTrigger, SpriteSheet, SpriteSheetAnimation},
    sound::DIRT_FOOTSTEPS_SOUND_PATH,
};

use super::super::UnitAnimation;

pub fn pikeman(world: &mut World) -> SpriteSheet<UnitAnimation> {
    let asset_server = world.resource::<AssetServer>();
    let texture: Handle<Image> = asset_server.load("sprites/humans/MiniSpearMan.png");
    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        7,
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
            first_sprite_index: 7,
            last_sprite_index: 12,
            ..default()
        },
        UnitAnimation::Attack => SpriteSheetAnimation {
            first_sprite_index: 21,
            last_sprite_index: 27,
            ..default()
        },
        UnitAnimation::Hit => SpriteSheetAnimation {
            first_sprite_index: 28,
            last_sprite_index: 30,
            ..default()
        },
        UnitAnimation::Death => SpriteSheetAnimation {
            first_sprite_index: 35,
            last_sprite_index: 39,
            ..default()
        },
    });

    let animations_sound = EnumMap::new(|c| match c {
        UnitAnimation::Idle => AnimationSound {
            sound_files: vec![],
            sound_trigger: AnimationSoundTrigger::OnStartFrameTimer,
        },
        UnitAnimation::Walk => AnimationSound {
            sound_files: vec![
                format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_1.ogg"),
                format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_2.ogg"),
                format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_3.ogg"),
                format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_4.ogg"),
                format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_5.ogg"),
                format!("{DIRT_FOOTSTEPS_SOUND_PATH}/dirt_footstep_6.ogg"),
            ],
            sound_trigger: AnimationSoundTrigger::OnStartFrameTimer,
        },
        UnitAnimation::Attack => AnimationSound {
            sound_files: vec![],
            sound_trigger: AnimationSoundTrigger::OnStartFrameTimer,
        },
        UnitAnimation::Hit => AnimationSound {
            sound_files: vec![],
            sound_trigger: AnimationSoundTrigger::OnStartFrameTimer,
        },
        UnitAnimation::Death => AnimationSound {
            sound_files: vec![],
            sound_trigger: AnimationSoundTrigger::OnStartFrameTimer,
        },
    });

    SpriteSheet {
        texture,
        layout,
        animations,
        animations_sound,
    }
}
