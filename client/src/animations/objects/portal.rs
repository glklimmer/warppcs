use bevy::prelude::*;

use shared::enum_map::*;

use crate::animations::{SpriteSheet, SpriteSheetAnimation};

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum PortalAnimation {
    #[default]
    Swirle,
}

#[derive(Resource)]
pub struct PortalSpriteSheet {
    pub sprite_sheet: SpriteSheet<PortalAnimation>,
}

impl FromWorld for PortalSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/objects/portal.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(32, 32),
            12,
            1,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            PortalAnimation::Swirle => SpriteSheetAnimation {
                first_sprite_index: 0,
                last_sprite_index: 11,
                ..default()
            },
        });

        PortalSpriteSheet {
            sprite_sheet: SpriteSheet {
                texture,
                layout,
                animations,
            },
        }
    }
}
