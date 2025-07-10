use bevy::prelude::*;

use shared::enum_map::*;

use crate::anim;
use crate::animations::AnimationSpriteSheet;

const ATLAS_COLUMNS: usize = 12;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum PortalAnimation {
    #[default]
    Swirle,
}

#[derive(Resource)]
pub struct PortalSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<PortalAnimation, Image>,
}

impl FromWorld for PortalSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture = asset_server.load("sprites/objects/portal.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(32, 32),
            ATLAS_COLUMNS as u32,
            1,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            PortalAnimation::Swirle => anim!(0, 11),
        });

        let animations_sound = EnumMap::new(|c| match c {
            PortalAnimation::Swirle => None,
        });

        PortalSpriteSheet {
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
