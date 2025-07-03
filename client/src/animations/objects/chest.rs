use bevy::prelude::*;
use shared::enum_map::*;

use shared::ChestAnimation;
use shared::ChestAnimationEvent;
use shared::server::players::chest::Chest;

use crate::animations::AnimationDirection;
use crate::animations::PlayOnce;

use super::super::{AnimationSpriteSheet, SpriteSheetAnimation};

#[derive(Resource)]
pub struct ChestSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<ChestAnimation, Image>,
}

impl FromWorld for ChestSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/objects/chest.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(32, 32),
            3,
            2,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            ChestAnimation::Open => SpriteSheetAnimation {
                first_sprite_index: 0,
                last_sprite_index: 2,
                ..default()
            },
            ChestAnimation::Close => SpriteSheetAnimation {
                first_sprite_index: 2,
                last_sprite_index: 0,
                direction: AnimationDirection::Backward,
                ..default()
            },
        });

        let animations_sound = EnumMap::new(|c| match c {
            ChestAnimation::Open => None,
            ChestAnimation::Close => None,
        });

        ChestSpriteSheet {
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

pub fn play_chest_animation(
    mut animation_changes: EventReader<ChestAnimationEvent>,
    mut commands: Commands,
    mut query: Query<&mut Sprite>,
    chest_sprite_sheet: Res<ChestSpriteSheet>,
) {
    for event in animation_changes.read() {
        let Ok(mut sprite) = query.get_mut(event.entity) else {
            continue;
        };

        let animation = chest_sprite_sheet
            .sprite_sheet
            .animations
            .get(event.animation);

        commands
            .entity(event.entity)
            .insert((PlayOnce, animation.clone()));

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = animation.first_sprite_index;
        }
    }
}

pub fn set_chest_after_play_once(
    trigger: Trigger<OnRemove, PlayOnce>,
    mut commands: Commands,
    chest: Query<&Chest>,
) {
    if let Ok(_) = chest.get(trigger.target()) {
        commands
            .entity(trigger.target())
            .remove::<SpriteSheetAnimation>();
    }
}
