use bevy::prelude::*;

use shared::enum_map::*;

use crate::{
    animations::{AnimationSound, AnimationSoundTrigger, SpriteSheet, SpriteSheetAnimation},
    sound::GRASS_FOOTSTEPS_SOUND_PATH,
};

use super::super::UnitAnimation;

pub fn commander(world: &mut World) -> SpriteSheet<UnitAnimation> {
    let asset_server = world.resource::<AssetServer>();
    let texture: Handle<Image> = asset_server.load("sprites/humans/MiniPrinceMan.png");

    let footstep1 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_1.ogg"));
    let footstep2 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_2.ogg"));
    let footstep3 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_3.ogg"));
    let footstep4 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_4.ogg"));
    let footstep5 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_5.ogg"));
    let footstep6 = asset_server.load(format!("{GRASS_FOOTSTEPS_SOUND_PATH}/grass_footstep_6.ogg"));

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
            ..default()
        },

        UnitAnimation::Walk => SpriteSheetAnimation {
            first_sprite_index: 11,
            last_sprite_index: 16,
            ..default()
        },
        UnitAnimation::Attack => SpriteSheetAnimation {
            first_sprite_index: 33,
            last_sprite_index: 42,
            ..default()
        },
        UnitAnimation::Hit => SpriteSheetAnimation {
            first_sprite_index: 55,
            last_sprite_index: 57,
            ..default()
        },
        UnitAnimation::Death => SpriteSheetAnimation {
            first_sprite_index: 66,
            last_sprite_index: 69,
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

    SpriteSheet {
        texture,
        layout,
        animations,
        animations_sound,
    }
}
