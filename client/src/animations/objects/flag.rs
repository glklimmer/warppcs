use bevy::prelude::*;

use shared::{enum_map::*, flag_collider, BoxCollider};

use crate::{
    animations::{AnimationSound, AnimationSoundTrigger, SpriteSheet, SpriteSheetAnimation},
    entities::{highlight::Highlightable, map::PartOfScene},
};

#[derive(Component)]
#[require(PartOfScene, FlagAnimation, BoxCollider(flag_collider), Highlightable)]
pub struct Flag;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum FlagAnimation {
    #[default]
    Wave,
}

#[derive(Resource)]
pub struct FlagSpriteSheet {
    pub sprite_sheet: SpriteSheet<FlagAnimation>,
}

impl FromWorld for FlagSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/objects/flag.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(48, 64),
            8,
            1,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            FlagAnimation::Wave => SpriteSheetAnimation {
                first_sprite_index: 0,
                last_sprite_index: 7,
                ..default()
            },
        });

        let animations_sound = EnumMap::new(|c| match c {
            FlagAnimation::Wave => AnimationSound {
                sound_files: vec![],
                sound_trigger: AnimationSoundTrigger::OnEnter,
            },
        });

        FlagSpriteSheet {
            sprite_sheet: SpriteSheet {
                texture,
                layout,
                animations,
                animations_sound,
            },
        }
    }
}
