use bevy::prelude::*;

use shared::enum_map::*;

use crate::animations::StaticSpriteSheet;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum ItemInfoParts {
    #[default]
    ItemPreview,
    Text,
}

#[derive(Resource)]
pub struct ItemInfoSpriteSheet {
    pub sprite_sheet: StaticSpriteSheet<ItemInfoParts>,
}

impl FromWorld for ItemInfoSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/ui/item_info.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout {
            size: UVec2 { x: 100, y: 68 },
            textures: vec![
                URect {
                    min: UVec2 { x: 0, y: 0 },
                    max: UVec2 { x: 100, y: 58 },
                },
                URect {
                    min: UVec2 { x: 0, y: 59 },
                    max: UVec2 { x: 100, y: 68 },
                },
            ],
        });

        let parts = EnumMap::new(|c| match c {
            ItemInfoParts::ItemPreview => 0,
            ItemInfoParts::Text => 1,
        });

        Self {
            sprite_sheet: StaticSpriteSheet {
                texture,
                layout,
                parts,
            },
        }
    }
}
