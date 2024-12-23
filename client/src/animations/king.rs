use bevy::prelude::*;

use shared::enum_map::*;

use super::{SpriteSheet, SpriteSheetAnimation};

#[derive(PartialEq, Eq, Debug, Clone, Copy, Mappable)]
pub enum KingAnimation {
    Idle,
    Drink,
    Walk,
    Attack,
}

#[derive(Resource)]
pub struct KingSpriteSheet {
    pub sprite_sheet: SpriteSheet<KingAnimation>,
}

impl FromWorld for KingSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/humans/Outline/MiniKingMan.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let idle = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            4,
            None,
            None,
        ));

        let drink = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            5,
            None,
            Some(UVec2::new(0, 32)),
        ));

        let walk = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            6,
            None,
            Some(UVec2::new(0, 32 * 2)),
        ));

        let attack = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            11,
            None,
            Some(UVec2::new(0, 32 * 3)),
        ));

        let animations = EnumMap::new(|c| match c {
            KingAnimation::Idle => SpriteSheetAnimation {
                layout: idle.clone(),
                first_sprite_index: 0,
                last_sprite_index: 3,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
            KingAnimation::Drink => SpriteSheetAnimation {
                layout: drink.clone(),
                first_sprite_index: 0,
                last_sprite_index: 4,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
            KingAnimation::Walk => SpriteSheetAnimation {
                layout: walk.clone(),
                first_sprite_index: 0,
                last_sprite_index: 5,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
            KingAnimation::Attack => SpriteSheetAnimation {
                layout: attack.clone(),
                first_sprite_index: 0,
                last_sprite_index: 10,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
        });

        KingSpriteSheet {
            sprite_sheet: SpriteSheet {
                texture,
                animations,
            },
        }
    }
}
