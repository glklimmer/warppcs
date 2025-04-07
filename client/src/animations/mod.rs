use bevy::prelude::*;

use animals::horse::{
    HorseAnimation, HorseSpriteSheet, next_horse_animation, set_horse_sprite_animation,
};
use bevy_replicon::client::ClientSet;
use king::{
    KingAnimation, KingSpriteSheet, set_king_after_play_once, set_king_idle,
    set_king_sprite_animation, set_king_walking, trigger_king_animation,
};
use objects::{
    chest::{ChestSpriteSheet, play_chest_animation, set_chest_after_play_once},
    flag::FlagSpriteSheet,
    items::weapons::WeaponsSpriteSheet,
    portal::PortalSpriteSheet,
};
use shared::{enum_map::*, server::entities::UnitAnimation};
use units::{
    UnitSpriteSheets, set_unit_after_play_once, set_unit_idle, set_unit_sprite_animation,
    set_unit_walking, trigger_unit_animation,
};

pub mod animals;
pub mod king;
pub mod objects;
pub mod units;

#[derive(Clone)]
pub struct SpriteSheet<E: EnumIter> {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub animations: EnumMap<E, SpriteSheetAnimation>,
    pub animations_sound: EnumMap<E, Option<AnimationSound>>,
}

#[derive(Clone)]
pub enum AnimationDirection {
    Forward,
    Backward,
}

#[derive(Component, Clone, Default, PartialEq, Eq)]
pub enum AnimationSoundTrigger {
    #[default]
    OnEnter,
    OnStartFrameTimer,
    OnEndFrameTimer,
}

#[derive(Component, Clone)]
#[require(AnimationSoundTrigger)]
pub struct AnimationSound {
    pub sound_handles: Vec<Handle<AudioSource>>,
    pub sound_trigger: AnimationSoundTrigger,
}

#[derive(Component, Clone)]
pub struct SpriteSheetAnimation {
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub frame_timer: Timer,
    pub direction: AnimationDirection,
}

impl Default for SpriteSheetAnimation {
    fn default() -> Self {
        SpriteSheetAnimation {
            first_sprite_index: 0,
            last_sprite_index: 0,
            frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            direction: AnimationDirection::Forward,
        }
    }
}

/// Gets only triggered if new animation
#[derive(Debug, Event)]
pub struct AnimationTrigger<E> {
    pub entity: Entity,
    pub state: E,
}

#[derive(Component)]
pub struct FullAnimation;

#[derive(Component)]
pub struct PlayOnce;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnitSpriteSheets>();
        app.add_event::<AnimationTrigger<UnitAnimation>>();

        app.init_resource::<KingSpriteSheet>();
        app.add_event::<AnimationTrigger<KingAnimation>>();

        app.init_resource::<FlagSpriteSheet>();
        app.init_resource::<ChestSpriteSheet>();
        app.init_resource::<PortalSpriteSheet>();
        app.init_resource::<WeaponsSpriteSheet>();

        app.init_resource::<HorseSpriteSheet>();
        app.add_event::<AnimationTrigger<HorseAnimation>>();

        app.add_systems(
            PreUpdate,
            (
                trigger_king_animation,
                trigger_unit_animation,
                play_chest_animation,
            )
                .after(ClientSet::Receive),
        )
        .add_observer(set_king_walking)
        .add_observer(set_king_idle)
        .add_observer(set_king_after_play_once)
        .add_observer(set_unit_walking)
        .add_observer(set_unit_idle)
        .add_observer(set_unit_after_play_once)
        .add_observer(set_chest_after_play_once);

        app.add_systems(
            Update,
            (
                (set_unit_sprite_animation),
                (set_king_sprite_animation),
                (set_horse_sprite_animation, next_horse_animation),
                advance_animation,
            ),
        );
    }
}

#[allow(clippy::type_complexity)]
fn advance_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut SpriteSheetAnimation,
        &mut Sprite,
        Option<&FullAnimation>,
        Option<&PlayOnce>,
    )>,
) {
    for (entity, mut animation, mut sprite, maybe_full, maybe_play_once) in &mut query {
        animation.frame_timer.tick(time.delta());
        let atlas = sprite.texture_atlas.as_mut().unwrap();

        if animation.frame_timer.just_finished() {
            atlas.index = if atlas.index == animation.last_sprite_index {
                if maybe_play_once.is_some() {
                    commands.entity(entity).remove::<PlayOnce>();
                    return;
                }
                if maybe_full.is_some() {
                    commands.entity(entity).remove::<FullAnimation>();
                }
                animation.first_sprite_index
            } else {
                match animation.direction {
                    AnimationDirection::Forward => atlas.index + 1,
                    AnimationDirection::Backward => atlas.index - 1,
                }
            };
        }
    }
}
