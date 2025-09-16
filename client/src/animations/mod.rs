use bevy::prelude::*;

use crate::{
    animations::{
        king::{set_king_defeat, set_king_defeat_play_once},
        ui::{
            animations::UIAnimationsPlugin, army_formations::FormationIconSpriteSheet,
            commander_menu::CommanderMenuSpriteSheet,
        },
    },
    asset_loader::AssetsToLoad,
};
use animals::horse::{
    HorseAnimation, HorseSpriteSheet, next_horse_animation, set_horse_sprite_animation,
};
use bevy_replicon::client::ClientSet;
use buildings::{BuildingSpriteSheets, remove_animation_after_play_once, update_building_sprite};
use king::{
    KingAnimation, KingSpriteSheet, set_king_after_play_once, set_king_idle,
    set_king_sprite_animation, set_king_walking, trigger_king_animation,
};
use objects::{
    chest::{ChestSpriteSheet, play_chest_animation, set_chest_after_play_once},
    flag::{FlagSpriteSheet, play_flag_animation},
    items::{
        chests::ChestsSpriteSheet, feet::FeetSpriteSheet, heads::HeadsSpriteSheet,
        weapons::WeaponsSpriteSheet,
    },
    portal::PortalSpriteSheet,
    projectiles::ProjectileSpriteSheet,
};
use shared::{enum_map::*, server::entities::UnitAnimation};

use sprite_variant_loader::{SpriteVariantLoader, SpriteVariants};
use ui::{item_info::ItemInfoSpriteSheet, map_icon::MapIconSpriteSheet};
use units::{
    UnitSpriteSheets, set_unit_after_play_once, set_unit_idle, set_unit_sprite_animation,
    set_unit_walking, trigger_unit_animation,
};

pub mod animals;
pub mod buildings;
pub mod king;
pub mod macros;
pub mod objects;
pub mod sprite_variant_loader;
pub mod ui;
pub mod units;

#[derive(Clone)]
pub struct StaticSpriteSheet<E: EnumIter> {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub parts: EnumMap<E, usize>,
}

impl<E: EnumIter> StaticSpriteSheet<E> {
    pub fn new(
        world: &mut World,
        texture: Handle<Image>,
        layout: Handle<TextureAtlasLayout>,
        parts: EnumMap<E, usize>,
    ) -> Self {
        let mut assets_to_load = world.resource_mut::<AssetsToLoad>();
        assets_to_load.push(texture.clone().untyped());

        Self {
            texture,
            layout,
            parts,
        }
    }

    pub fn texture_atlas(&self, part: E) -> TextureAtlas {
        TextureAtlas {
            layout: self.layout.clone(),
            index: *self.parts.get(part),
        }
    }
}

#[derive(Clone)]
pub struct AnimationSpriteSheet<E: EnumIter, T: Asset> {
    pub texture: Handle<T>,
    pub layout: Handle<TextureAtlasLayout>,
    pub animations: EnumMap<E, SpriteSheetAnimation>,
    pub animations_sound: EnumMap<E, Option<AnimationSound>>,
}

impl<E: EnumIter, T: Asset> AnimationSpriteSheet<E, T> {
    pub fn new(
        world: &mut World,
        texture: Handle<T>,
        layout: Handle<TextureAtlasLayout>,
        animations: EnumMap<E, SpriteSheetAnimation>,
        animations_sound: EnumMap<E, Option<AnimationSound>>,
    ) -> Self {
        let mut assets_to_load = world.resource_mut::<AssetsToLoad>();
        assets_to_load.push(texture.clone().untyped());

        for sound in animations_sound.iter().flatten() {
            for handle in &sound.sound_handles {
                assets_to_load.push(handle.clone().untyped());
            }
        }

        Self {
            texture,
            layout,
            animations,
            animations_sound,
        }
    }
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

impl SpriteSheetAnimation {
    pub fn with_total_duration(&mut self, total_seconds: f32) {
        let frame_count = (self.last_sprite_index - self.first_sprite_index + 1) as f32;
        let per_frame = total_seconds / frame_count;
        self.frame_timer = Timer::from_seconds(per_frame, TimerMode::Repeating);
    }
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
pub struct PlayOnce;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UIAnimationsPlugin);
        app.init_asset::<SpriteVariants>();
        app.init_asset_loader::<SpriteVariantLoader>();

        app.init_resource::<UnitSpriteSheets>();
        app.add_event::<AnimationTrigger<UnitAnimation>>();

        app.init_resource::<KingSpriteSheet>();
        app.add_event::<AnimationTrigger<KingAnimation>>();

        app.init_resource::<FlagSpriteSheet>();
        app.init_resource::<ChestSpriteSheet>();
        app.init_resource::<PortalSpriteSheet>();
        app.init_resource::<WeaponsSpriteSheet>();
        app.init_resource::<ChestsSpriteSheet>();
        app.init_resource::<HeadsSpriteSheet>();
        app.init_resource::<FeetSpriteSheet>();
        app.init_resource::<ProjectileSpriteSheet>();
        app.init_resource::<CommanderMenuSpriteSheet>();

        app.init_resource::<HorseSpriteSheet>();
        app.add_event::<AnimationTrigger<HorseAnimation>>();

        app.init_resource::<ItemInfoSpriteSheet>();
        app.init_resource::<MapIconSpriteSheet>();
        app.init_resource::<FormationIconSpriteSheet>();

        app.init_resource::<BuildingSpriteSheets>();

        app.add_systems(
            PreUpdate,
            (
                trigger_king_animation,
                trigger_unit_animation,
                play_chest_animation,
                play_flag_animation,
            )
                .after(ClientSet::Receive),
        )
        .add_observer(set_king_defeat)
        .add_observer(set_king_defeat_play_once)
        .add_observer(set_king_walking)
        .add_observer(set_king_idle)
        .add_observer(set_king_after_play_once)
        .add_observer(set_unit_walking)
        .add_observer(set_unit_idle)
        .add_observer(set_unit_after_play_once)
        .add_observer(set_chest_after_play_once)
        .add_observer(remove_animation_after_play_once);

        app.add_systems(
            Update,
            (
                (
                    set_unit_sprite_animation,
                    set_king_sprite_animation,
                    set_horse_sprite_animation,
                    next_horse_animation,
                    update_building_sprite,
                ),
                advance_animation,
            )
                .chain(),
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
        Option<&PlayOnce>,
    )>,
) {
    for (entity, mut animation, mut sprite, maybe_play_once) in &mut query {
        animation.frame_timer.tick(time.delta());
        let atlas = sprite.texture_atlas.as_mut().unwrap();

        if animation.frame_timer.just_finished() {
            atlas.index = if atlas.index == animation.last_sprite_index {
                if maybe_play_once.is_some() {
                    commands.entity(entity).remove::<PlayOnce>();
                    continue;
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
