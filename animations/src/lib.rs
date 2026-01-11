use bevy::prelude::*;

use shared::enum_map::*;
use sprite_variant_loader::{AssetsToLoad, SpriteVariantLoaderPlugin};
use ui::{item_info::ItemInfoSpriteSheet, map_icon::MapIconSpriteSheet};

use crate::{
    sound::{AnimationSound, AnimationSoundPlugin},
    ui::{
        animations::UIAnimationsPlugin, army_formations::FormationIconSpriteSheet,
        commander_menu::CommanderMenuSpriteSheet,
    },
};

pub use sprite_variant_loader::loader::{SpriteVariants, SpriteVariantsAssetsExt};

mod macros;

pub mod sound;
pub mod ui;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            UIAnimationsPlugin,
            SpriteVariantLoaderPlugin,
            AnimationSoundPlugin,
        ));

        app.init_resource::<CommanderMenuSpriteSheet>();

        app.init_resource::<ItemInfoSpriteSheet>();
        app.init_resource::<MapIconSpriteSheet>();
        app.init_resource::<FormationIconSpriteSheet>();

        app.add_systems(PostUpdate, advance_animation);
    }
}

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

pub trait BuildSprite<K> {
    fn sprite_for<T: Into<K>>(&self, kind: T) -> Sprite;
}

impl<K> BuildSprite<K> for AnimationSpriteSheet<K, Image>
where
    K: EnumIter,
{
    fn sprite_for<T: Into<K>>(&self, kind: T) -> Sprite {
        let animation = kind.into();
        let sprite_animation = self.animations.get(animation);
        Sprite {
            image: self.texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: self.layout.clone(),
                index: sprite_animation.first_sprite_index,
            }),
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub enum AnimationDirection {
    Forward,
    Backward,
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
#[derive(Debug, Message)]
pub struct AnimationTrigger<E> {
    pub entity: Entity,
    pub state: E,
}

#[derive(Component)]
pub struct PlayOnce;

#[allow(clippy::type_complexity)]
fn advance_animation(
    mut query: Query<(
        Entity,
        &mut SpriteSheetAnimation,
        &mut Sprite,
        Option<&PlayOnce>,
    )>,
    mut commands: Commands,
    time: Res<Time>,
) -> Result {
    for (entity, mut animation, mut sprite, maybe_play_once) in &mut query {
        animation.frame_timer.tick(time.delta());
        let atlas = sprite.texture_atlas.as_mut().ok_or(
            "No texture atlas for sprite animation found. Texture atlas needed for animations.",
        )?;

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
    Ok(())
}
