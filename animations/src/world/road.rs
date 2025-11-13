use bevy::prelude::*;

use shared::enum_map::*;

use crate::anim;
use crate::AnimationSpriteSheet;

const ATLAS_COLUMNS: usize = 1;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum RoadAnimation {
    #[default]
    Idle,
}

#[derive(Resource)]
pub struct RoadSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<RoadAnimation, Image>,
}

impl FromWorld for RoadSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture = asset_server.load("sprites/world/road.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(64, 32),
            ATLAS_COLUMNS as u32,
            1,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            RoadAnimation::Idle => anim!(0, 0),
        });

        let animations_sound = EnumMap::new(|c| match c {
            RoadAnimation::Idle => None,
        });

        RoadSpriteSheet {
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
