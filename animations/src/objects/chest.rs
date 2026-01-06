use bevy::prelude::*;

use interactables::chest::{Chest, ChestOpened};
use interaction::Interactable;
use shared::enum_map::*;

use crate::{AnimationSpriteSheet, PlayOnce, SpriteSheetAnimation, anim, anim_reverse};

const ATLAS_COLUMNS: usize = 3;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable)]
pub enum ChestAnimation {
    Open,
    Close,
}

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
            ATLAS_COLUMNS as u32,
            2,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            ChestAnimation::Open => anim!(0, 2),
            ChestAnimation::Close => anim_reverse!(0, 2),
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

pub fn on_chest_opened(
    trigger: On<Insert, ChestOpened>,
    mut query: Query<&mut Sprite>,
    chest_sprite_sheet: Res<ChestSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.entity;
    let mut sprite = query.get_mut(entity)?;

    let sprite_sheet_animation = chest_sprite_sheet
        .sprite_sheet
        .animations
        .get(ChestAnimation::Open);

    commands
        .entity(entity)
        .insert((PlayOnce, sprite_sheet_animation.clone()))
        .try_remove::<Interactable>();

    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = sprite_sheet_animation.first_sprite_index;
    }
    Ok(())
}

pub fn set_chest_after_play_once(
    trigger: On<Remove, PlayOnce>,
    chest: Query<Option<&Chest>>,
    mut commands: Commands,
) -> Result {
    if chest.get(trigger.entity)?.is_some() {
        commands
            .entity(trigger.entity)
            .remove::<SpriteSheetAnimation>();
    }
    Ok(())
}
