use bevy::prelude::*;

use enum_map::Mappable;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArcherSpriteSheet>();
    }
}

#[derive(Resource)]
pub struct ArcherSpriteSheet {
    pub texture: Handle<Image>,
    pub animations: EnumMap<Animation, AnimationConfig>,
}

#[derive(Copy, Clone, Mappable)]
pub enum Animation {
    Idle,
    Walk,
    Attack,
    Hit,
    Death,
}

#[derive(Component, Debug, Clone)]
pub struct AnimationConfig {
    pub layout_handle: Handle<TextureAtlasLayout>,
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub frame_timer: Timer,
}

impl FromWorld for ArcherSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture_handle: Handle<Image> =
            asset_server.load("sprites/humans/Outline/MiniArcherMan.png");

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let idle = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            4,
            None,
            None,
        ));

        let walk = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            6,
            None,
            Some(UVec2::new(0, 32)),
        ));

        let attack = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            11,
            None,
            Some(UVec2::new(32 * 3, 0)),
        ));

        let animations = EnumMap::new(|c| match c {
            Animation::Idle => AnimationConfig {
                layout_handle: idle.clone(),
                first_sprite_index: 0,
                last_sprite_index: 4,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
            Animation::Walk => AnimationConfig {
                layout_handle: walk.clone(),
                first_sprite_index: 0,
                last_sprite_index: 6,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
            Animation::Attack => AnimationConfig {
                layout_handle: attack.clone(),
                first_sprite_index: 0,
                last_sprite_index: 11,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
            Animation::Hit => todo!(),
            Animation::Death => todo!(),
        });

        ArcherSpriteSheet {
            texture: texture_handle,
            animations,
        }
    }
}

// impl EnumIter for Animation {
//     const COUNT: usize = 5;
//
//     fn all_variants() -> &'static [Self] {
//         static ALL: [Animation; 5] = [
//             Animation::Idle,
//             Animation::Walk,
//             Animation::Attack,
//             Animation::Hit,
//             Animation::Death,
//         ];
//         &ALL
//     }
//
//     fn as_index(&self) -> usize {
//         match *self {
//             Animation::Idle => 0,
//             Animation::Walk => 1,
//             Animation::Attack => 2,
//             Animation::Hit => 3,
//             Animation::Death => 4,
//         }
//     }
// }
