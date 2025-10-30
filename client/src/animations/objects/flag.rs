use bevy::prelude::*;

use serde::{Deserialize, Serialize};
use shared::{enum_map::*, server::players::flag::FlagDestroyed};

use crate::{
    anim,
    animations::{AnimationSpriteSheet, sprite_variant_loader::SpriteVariants},
    entities::highlight::Highlighted,
};

const ATLAS_COLUMNS: usize = 8;

#[derive(
    Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default, Deserialize, Serialize,
)]
pub enum FlagAnimation {
    #[default]
    Wave,
    Destroyed,
}

#[derive(Resource)]
pub struct FlagSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<FlagAnimation, SpriteVariants>,
}

impl FromWorld for FlagSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture = asset_server.load("sprites/objects/flag.png");

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(24, 24),
            ATLAS_COLUMNS as u32,
            2,
            Some(UVec2::splat(1)),
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            FlagAnimation::Wave => anim!(0, 7),
            FlagAnimation::Destroyed => anim!(1, 7),
        });

        let animations_sound = EnumMap::new(|c| match c {
            FlagAnimation::Wave => None,
            FlagAnimation::Destroyed => None,
        });

        FlagSpriteSheet {
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

pub fn on_flag_destroyed(
    trigger: Trigger<OnAdd, FlagDestroyed>,
    mut query: Query<&mut Sprite>,
    flag_sprite_sheet: Res<FlagSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.target();
    let mut sprite = query.get_mut(entity)?;

    let animation = flag_sprite_sheet
        .sprite_sheet
        .animations
        .get(FlagAnimation::Destroyed);

    commands
        .entity(entity)
        .insert(animation.clone())
        .remove::<Highlighted>();

    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = animation.first_sprite_index;
    }
    Ok(())
}
