use bevy::prelude::*;

use serde::{Deserialize, Serialize};
use shared::{
    Player,
    enum_map::*,
    server::{physics::attachment::AttachedTo, players::flag::FlagDestroyed},
};
use sprite_variant_loader::loader::SpriteVariants;

use crate::{anim, animations::AnimationSpriteSheet, entities::highlight::Highlighted};

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

// TODO: Change to Observer after Replcion 0.34 update
pub fn update_flag_visibility(
    flag: Query<(Entity, &AttachedTo), Changed<AttachedTo>>,
    player: Query<&Player>,
    mut commands: Commands,
) {
    for (flag, attachted_to) in flag.iter() {
        if player.get(**attachted_to).is_ok() {
            commands.entity(flag).insert(Visibility::Visible);
        } else {
            commands.entity(flag).insert(Visibility::Hidden);
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
