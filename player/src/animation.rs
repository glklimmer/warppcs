use bevy::prelude::*;

use animations::{
    AnimationSpriteSheet, AnimationTrigger, PlayOnce, SpriteSheetAnimation, SpriteVariants,
    SpriteVariantsAssetsExt, anim, anim_reverse, sound::AnimationSound,
    sound::AnimationSoundTrigger,
};
use bevy_replicon::client::ClientSystems;
use lobby::PlayerColor;
use mounts::Mounted;
use physics::movement::Moving;
use shared::{AnimationChange, AnimationChangeEvent, enum_map::*};

use crate::{Player, defeat::PlayerDefeated};

const ATLAS_COLUMNS: usize = 11;

pub(crate) struct PlayerAnimationPlugin;

impl Plugin for PlayerAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KingSpriteSheet>()
            .add_message::<AnimationTrigger<KingAnimation>>()
            .add_systems(
                PreUpdate,
                trigger_king_animation.after(ClientSystems::Receive),
            )
            .add_observer(init_player_sprite)
            .add_observer(set_king_defeat)
            .add_observer(remove_animation)
            .add_observer(set_king_walking)
            .add_observer(set_king_idle)
            .add_observer(set_king_after_play_once)
            .add_systems(Update, set_king_sprite_animation);
    }
}

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub(crate) enum KingAnimation {
    #[default]
    Idle,
    Drink,
    Walk,
    Attack,
    Hit,
    Death,
    KnockOut,
    Mount,
    Unmount,
    HorseIdle,
    HorseWalk,
}

#[derive(Resource)]
struct KingSpriteSheet {
    sprite_sheet: AnimationSpriteSheet<KingAnimation, SpriteVariants>,
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
            ATLAS_COLUMNS as u32,
            10,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            KingAnimation::Idle => anim!(0, 3),
            KingAnimation::Drink => anim!(1, 5),
            KingAnimation::Walk => anim!(2, 5),
            KingAnimation::Attack => anim!(4, 10),
            KingAnimation::Hit => anim!(5, 3),
            KingAnimation::Death => anim!(6, 6),
            KingAnimation::KnockOut => SpriteSheetAnimation {
                first_sprite_index: 6 * ATLAS_COLUMNS + 6,
                last_sprite_index: 6 * ATLAS_COLUMNS + 6,
                ..default()
            },
            KingAnimation::Mount => anim!(7, 6),
            KingAnimation::Unmount => anim_reverse!(7, 6),
            KingAnimation::HorseIdle => anim!(8, 7),
            KingAnimation::HorseWalk => anim!(9, 5),
        });

        let animations_sound = EnumMap::new(move |c| match c {
            KingAnimation::Idle => None,
            KingAnimation::Drink => None,
            KingAnimation::Walk => Some(AnimationSound {
                sound_handles: vec![walk_sound.clone()],
                sound_trigger: AnimationSoundTrigger::StartFrameTimer,
            }),
            KingAnimation::Attack => None,
            KingAnimation::Hit => None,
            KingAnimation::Death => None,
            KingAnimation::KnockOut => None,
            KingAnimation::Mount => Some(AnimationSound {
                sound_handles: vec![horse_sound.clone()],
                sound_trigger: AnimationSoundTrigger::Enter,
            }),
            KingAnimation::Unmount => Some(AnimationSound {
                sound_handles: vec![horse_sound.clone()],
                sound_trigger: AnimationSoundTrigger::Enter,
            }),
            KingAnimation::HorseIdle => None,
            KingAnimation::HorseWalk => Some(AnimationSound {
                sound_handles: vec![horse_gallop.clone()],
                sound_trigger: AnimationSoundTrigger::StartFrameTimer,
            }),
        });

        KingSpriteSheet {
            sprite_sheet: AnimationSpriteSheet::new(
                world,
                texture,
                layout,
                animations,
                animations_sound,
            ),
        }
    }
}

fn init_player_sprite(
    trigger: On<Add, Player>,
    mut players: Query<(&mut Sprite, &PlayerColor)>,
    king_sprite_sheet: Res<KingSpriteSheet>,
    variants: Res<Assets<SpriteVariants>>,
    mut commands: Commands,
) -> Result {
    let (mut sprite, color) = players.get_mut(trigger.entity)?;

    let handle = &king_sprite_sheet.sprite_sheet.texture;
    let sprite_variants = variants.get_variant(handle)?;
    let animation = king_sprite_sheet
        .sprite_sheet
        .animations
        .get(KingAnimation::Idle);

    sprite.image = sprite_variants.variants.get(*color).clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: king_sprite_sheet.sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((animation.clone(), KingAnimation::default()));
    Ok(())
}

fn trigger_king_animation(
    mut animation_changes: MessageReader<AnimationChangeEvent>,
    mut animation_trigger: MessageWriter<AnimationTrigger<KingAnimation>>,
    mounted: Query<Option<&Mounted>, With<Player>>,
    mut commands: Commands,
) {
    for event in animation_changes.read() {
        if let Ok(maybe_mounted) = mounted.get(event.entity) {
            let new_animation = match maybe_mounted {
                Some(_) => match &event.change {
                    AnimationChange::Idle => KingAnimation::Mount,
                    AnimationChange::Attack => KingAnimation::Mount,
                    AnimationChange::Hit(_) => KingAnimation::Mount,
                    AnimationChange::Death => KingAnimation::Death,
                    AnimationChange::KnockOut => KingAnimation::KnockOut,
                    AnimationChange::Mount => KingAnimation::Mount,
                    AnimationChange::Unmount => KingAnimation::Unmount,
                },
                None => match &event.change {
                    AnimationChange::Idle => KingAnimation::Idle,
                    AnimationChange::Attack => KingAnimation::Attack,
                    AnimationChange::Hit(_) => KingAnimation::Hit,
                    AnimationChange::Death => KingAnimation::Death,
                    AnimationChange::KnockOut => KingAnimation::KnockOut,
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

fn set_king_walking(
    trigger: On<Add, Moving>,
    mut animation_trigger: MessageWriter<AnimationTrigger<KingAnimation>>,
    mounted: Query<Option<&Mounted>, With<Player>>,
) {
    if let Ok(maybe_mounted) = mounted.get(trigger.entity) {
        let new_animation = match maybe_mounted {
            Some(_) => KingAnimation::HorseWalk,
            None => KingAnimation::Walk,
        };

        animation_trigger.write(AnimationTrigger {
            entity: trigger.entity,
            state: new_animation,
        });
    }
}

fn set_king_defeat(
    trigger: On<PlayerDefeated>,
    mut animation_trigger: MessageWriter<AnimationTrigger<KingAnimation>>,
    mut commands: Commands,
) {
    commands.entity(**trigger).insert(PlayOnce);
    animation_trigger.write(AnimationTrigger {
        entity: **trigger,
        state: KingAnimation::Death,
    });
}

fn remove_animation(
    trigger: On<Remove, PlayOnce>,
    current_animation: Query<&KingAnimation>,
    mut commands: Commands,
) {
    if let Ok(KingAnimation::Death) = current_animation.get(trigger.entity) {
        commands
            .entity(trigger.entity)
            .remove::<SpriteSheetAnimation>();
    };
}

fn set_king_after_play_once(
    trigger: On<Remove, PlayOnce>,
    mut animation_trigger: MessageWriter<AnimationTrigger<KingAnimation>>,
    mounted: Query<(&KingAnimation, Option<&Mounted>)>,
) {
    if let Ok((animation, maybe_mounted)) = mounted.get(trigger.entity) {
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
            entity: trigger.entity,
            state: new_animation,
        });
    }
}

fn set_king_idle(
    trigger: On<Remove, Moving>,
    mut animation_trigger: MessageWriter<AnimationTrigger<KingAnimation>>,
    mounted: Query<Option<&Mounted>, With<Player>>,
) {
    if let Ok(maybe_mounted) = mounted.get(trigger.entity) {
        let new_animation = match maybe_mounted {
            Some(_) => KingAnimation::HorseIdle,
            None => KingAnimation::Idle,
        };

        animation_trigger.write(AnimationTrigger {
            entity: trigger.entity,
            state: new_animation,
        });
    }
}

fn set_king_sprite_animation(
    mut query: Query<(
        Entity,
        &mut SpriteSheetAnimation,
        &mut Sprite,
        &mut KingAnimation,
    )>,
    mut animation_changed: MessageReader<AnimationTrigger<KingAnimation>>,
    king_sprite_sheet: Res<KingSpriteSheet>,
    mut command: Commands,
) -> Result {
    for new_animation in animation_changed.read() {
        let (entity, mut sprite_animation, mut sprite, mut current_animation) =
            query.get_mut(new_animation.entity)?;
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
    Ok(())
}
