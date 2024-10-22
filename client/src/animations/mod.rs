use bevy::prelude::*;

use animation::AnimationPlugin;
use king::{AnimationConfig, KingPlugin};

pub mod animation;
pub mod king;

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimationPlugin);

        app.add_plugins(KingPlugin);
    }
}

#[derive(Resource)]
pub struct FlagSpriteSheet {
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for FlagSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture_handle: Handle<Image> = asset_server.load("aseprite/flag.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::new(48, 64), 8, 1, None, None);

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        FlagSpriteSheet {
            texture: texture_handle,
            atlas_layout: texture_atlas_layouts.add(layout),
        }
    }
}

#[derive(Component, Debug)]
pub struct Flag;

#[derive(Bundle, Debug)]
pub struct FlagBundle {
    pub flag: Flag,
    pub sprite_sheet: SpriteBundle,
    pub texture_atlas: TextureAtlas,
    pub animation_config: AnimationConfig,
}

impl FlagBundle {
    pub fn new(flag_sprite_sheet: &Res<FlagSpriteSheet>, translation: [f32; 3]) -> Self {
        let atlas_layout = flag_sprite_sheet.atlas_layout.clone();

        FlagBundle {
            flag: Flag,
            sprite_sheet: SpriteBundle {
                texture: flag_sprite_sheet.texture.clone(),
                transform: Transform {
                    translation: Vec3::new(translation[0], translation[1], translation[2]),
                    scale: Vec3::splat(2.),
                    ..Default::default()
                },
                ..Default::default()
            },
            texture_atlas: TextureAtlas {
                layout: atlas_layout.clone(),
                index: 7,
            },
            animation_config: AnimationConfig {
                layout_handle: atlas_layout,
                first_sprite_index: 7,
                last_sprite_index: 0,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
        }
    }
}
