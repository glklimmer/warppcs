use bevy::prelude::*;

use animations::{AnimationSpriteSheet, PlayOnce, SpriteSheetAnimation, anim, anim_reverse};
use interaction::Interactable;
use shared::enum_map::*;

use crate::chest::{Chest, ChestOpened};

pub(crate) struct ChestAnimationPlugin;

impl Plugin for ChestAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChestSpriteSheet>()
            .add_observer(on_chest_opened)
            .add_observer(set_chest_after_play_once)
            .add_observer(init_chest_sprite);
    }
}

const ATLAS_COLUMNS: usize = 3;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable)]
enum ChestAnimation {
    Open,
    Close,
}

#[derive(Resource)]
struct ChestSpriteSheet {
    sprite_sheet: AnimationSpriteSheet<ChestAnimation, Image>,
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

fn init_chest_sprite(
    trigger: On<Add, Chest>,
    mut chests: Query<&mut Sprite>,
    sprite_sheets: Res<ChestSpriteSheet>,
) -> Result {
    let mut sprite = chests.get_mut(trigger.entity)?;

    let sprite_sheet = &sprite_sheets.sprite_sheet;
    let animation = sprite_sheet.animations.get(ChestAnimation::Open);

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });
    Ok(())
}

fn on_chest_opened(
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

fn set_chest_after_play_once(
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
