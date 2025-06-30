use bevy::prelude::*;

use shared::{
    AnimationChange, AnimationChangeEvent, enum_map::*, networking::Mounted,
    server::physics::movement::Moving,
};

use crate::animations::AnimationDirection;

use super::{
    AnimationSound, AnimationSoundTrigger, AnimationSpriteSheet, AnimationTrigger, PlayOnce,
    SpriteSheetAnimation, sprite_variant_loader::SpriteVariants,
};

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum KingAnimation {
    #[default]
    Idle,
    Drink,
    Walk,
    Attack,
    Hit,
    Death,
    Mount,
    Unmount,
    HorseIdle,
    HorseWalk,
}

#[derive(Resource)]
pub struct KingSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<KingAnimation, SpriteVariants>,
}

impl FromWorld for KingSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let texture = asset_server.load("sprites/humans/MiniKingMan.png");

        let walk_sound = asset_server.load("animation_sound/king/walk.ogg");
        let horse_sound = asset_server.load("animation_sound/horse/horse_sound.ogg");
        let horse_gallop = asset_server.load("animation_sound/horse/horse_gallop.ogg");

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            10,
            10,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            KingAnimation::Idle => SpriteSheetAnimation {
                first_sprite_index: 0,
                last_sprite_index: 3,
                ..default()
            },
            KingAnimation::Drink => SpriteSheetAnimation {
                first_sprite_index: 10,
                last_sprite_index: 14,
                ..default()
            },
            KingAnimation::Walk => SpriteSheetAnimation {
                first_sprite_index: 20,
                last_sprite_index: 25,
                ..default()
            },
            KingAnimation::Attack => SpriteSheetAnimation {
                first_sprite_index: 40,
                last_sprite_index: 49,
                ..default()
            },
            KingAnimation::Hit => SpriteSheetAnimation {
                first_sprite_index: 50,
                last_sprite_index: 52,
                ..default()
            },
            KingAnimation::Death => SpriteSheetAnimation {
                first_sprite_index: 60,
                last_sprite_index: 65,
                ..default()
            },
            KingAnimation::Mount => SpriteSheetAnimation {
                first_sprite_index: 70,
                last_sprite_index: 76,
                ..default()
            },
            KingAnimation::Unmount => SpriteSheetAnimation {
                first_sprite_index: 76,
                last_sprite_index: 70,
                direction: AnimationDirection::Backward,
                ..default()
            },
            KingAnimation::HorseIdle => SpriteSheetAnimation {
                first_sprite_index: 80,
                last_sprite_index: 87,
                ..default()
            },
            KingAnimation::HorseWalk => SpriteSheetAnimation {
                first_sprite_index: 90,
                last_sprite_index: 95,
                ..default()
            },
        });

        let animations_sound = EnumMap::new(move |c| match c {
            KingAnimation::Idle => None,
            KingAnimation::Drink => None,
            KingAnimation::Walk => Some(AnimationSound {
                sound_handles: vec![walk_sound.clone()],
                sound_trigger: AnimationSoundTrigger::OnStartFrameTimer,
            }),
            KingAnimation::Attack => None,
            KingAnimation::Hit => None,
            KingAnimation::Death => None,
            KingAnimation::Mount => Some(AnimationSound {
                sound_handles: vec![horse_sound.clone()],
                sound_trigger: AnimationSoundTrigger::OnEnter,
            }),
            KingAnimation::Unmount => Some(AnimationSound {
                sound_handles: vec![horse_sound.clone()],
                sound_trigger: AnimationSoundTrigger::OnEnter,
            }),
            KingAnimation::HorseIdle => None,
            KingAnimation::HorseWalk => Some(AnimationSound {
                sound_handles: vec![horse_gallop.clone()],
                sound_trigger: AnimationSoundTrigger::OnStartFrameTimer,
            }),
        });

        KingSpriteSheet {
            sprite_sheet: AnimationSpriteSheet {
                texture,
                layout,
                animations,
                animations_sound,
            },
        }
    }
}

pub fn trigger_king_animation(
    mut animation_changes: EventReader<AnimationChangeEvent>,
    mut animation_trigger: EventWriter<AnimationTrigger<KingAnimation>>,
    mut commands: Commands,
    mounted: Query<Option<&Mounted>>,
) {
    for event in animation_changes.read() {
        if let Ok(maybe_mounted) = mounted.get(event.entity) {
            let new_animation = match maybe_mounted {
                Some(_) => match &event.change {
                    AnimationChange::Attack => todo!(),
                    AnimationChange::Hit(_) => todo!(),
                    AnimationChange::Death => todo!(),
                    AnimationChange::Mount => KingAnimation::Mount,
                    AnimationChange::Unmount => KingAnimation::Unmount,
                },
                None => match &event.change {
                    AnimationChange::Attack => KingAnimation::Attack,
                    AnimationChange::Hit(_) => KingAnimation::Hit,
                    AnimationChange::Death => KingAnimation::Death,
                    AnimationChange::Mount => KingAnimation::Mount,
                    AnimationChange::Unmount => KingAnimation::Unmount,
                },
            };

            commands.entity(event.entity).insert(PlayOnce);

            animation_trigger.write(AnimationTrigger {
                entity: event.entity,
                state: new_animation,
            });
        }
    }
}

pub fn set_king_walking(
    trigger: Trigger<OnAdd, Moving>,
    mut animation_trigger: EventWriter<AnimationTrigger<KingAnimation>>,
    mounted: Query<Option<&Mounted>>,
) {
    if let Ok(maybe_mounted) = mounted.get(trigger.target()) {
        let new_animation = match maybe_mounted {
            Some(_) => KingAnimation::HorseWalk,
            None => KingAnimation::Walk,
        };

        animation_trigger.write(AnimationTrigger {
            entity: trigger.target(),
            state: new_animation,
        });
    }
}

pub fn set_king_after_play_once(
    trigger: Trigger<OnRemove, PlayOnce>,
    mut animation_trigger: EventWriter<AnimationTrigger<KingAnimation>>,
    mounted: Query<(&KingAnimation, Option<&Mounted>)>,
) {
    if let Ok((animation, maybe_mounted)) = mounted.get(trigger.target()) {
        let new_animation = match animation {
            KingAnimation::Attack | KingAnimation::Mount | KingAnimation::Unmount => {
                match maybe_mounted {
                    Some(_) => KingAnimation::HorseIdle,
                    None => KingAnimation::Idle,
                }
            }
            _ => *animation,
        };

        animation_trigger.write(AnimationTrigger {
            entity: trigger.target(),
            state: new_animation,
        });
    }
}

pub fn set_king_idle(
    trigger: Trigger<OnRemove, Moving>,
    mut animation_trigger: EventWriter<AnimationTrigger<KingAnimation>>,
    mounted: Query<Option<&Mounted>>,
) {
    if let Ok(maybe_mounted) = mounted.get(trigger.target()) {
        let new_animation = match maybe_mounted {
            Some(_) => KingAnimation::HorseIdle,
            None => KingAnimation::Idle,
        };

        animation_trigger.write(AnimationTrigger {
            entity: trigger.target(),
            state: new_animation,
        });
    }
}

pub fn set_king_sprite_animation(
    mut command: Commands,
    mut query: Query<(
        Entity,
        &mut SpriteSheetAnimation,
        &mut Sprite,
        &mut KingAnimation,
    )>,
    mut animation_changed: EventReader<AnimationTrigger<KingAnimation>>,
    king_sprite_sheet: Res<KingSpriteSheet>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((entity, mut sprite_animation, mut sprite, mut current_animation)) =
            query.get_mut(new_animation.entity)
        {
            let animation = king_sprite_sheet
                .sprite_sheet
                .animations
                .get(new_animation.state);

            let sound = king_sprite_sheet
                .sprite_sheet
                .animations_sound
                .get(new_animation.state);

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = animation.first_sprite_index;
            }

            match sound {
                Some(sound) => {
                    command.entity(entity).insert(sound.clone());
                }
                None => {
                    command.entity(entity).remove::<AnimationSound>();
                }
            }

            *sprite_animation = animation.clone();
            *current_animation = new_animation.state;
        }
    }
}
