use bevy::prelude::*;

use shared::{
    enum_map::*,
    server::players::{
        chest::{Chest, ChestOpened},
        interaction::Interactable,
    },
};

use crate::{
    anim, anim_reverse,
    animations::{AnimationSpriteSheet, PlayOnce, SpriteSheetAnimation},
};

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
    trigger: Trigger<OnInsert, ChestOpened>,
    mut commands: Commands,
    mut query: Query<&mut Sprite>,
    chest_sprite_sheet: Res<ChestSpriteSheet>,
) {
    let entity = trigger.target();
    let Ok(mut sprite) = query.get_mut(entity) else {
        return;
    };

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
}

pub fn set_chest_after_play_once(
    trigger: Trigger<OnRemove, PlayOnce>,
    mut commands: Commands,
    chest: Query<&Chest>,
) {
    if chest.get(trigger.target()).is_ok() {
        commands
            .entity(trigger.target())
            .remove::<SpriteSheetAnimation>();
    }
}
