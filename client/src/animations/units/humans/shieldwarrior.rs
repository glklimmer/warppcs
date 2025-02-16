use bevy::prelude::*;

use shared::enum_map::*;

use crate::animations::{AnimationSound, AnimationSoundTrigger, SpriteSheet, SpriteSheetAnimation};

use super::super::UnitAnimation;

pub fn shieldwarrior(world: &mut World) -> SpriteSheet<UnitAnimation> {
    let asset_server = world.resource::<AssetServer>();
    let texture: Handle<Image> = asset_server.load("sprites/humans/MiniShieldMan.png");
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
            animation_sound: Some(AnimationSound {
                sound_file: "animation_sound/shieldwarrior/sword_hit.ogg".to_string(),
                sound_trigger: AnimationSoundTrigger::OnEndFrameTimer,
            }),
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

    SpriteSheet {
        texture,
        layout,
        animations,
    }
}
