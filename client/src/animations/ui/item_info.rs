use bevy::prelude::*;

#[derive(Resource)]
pub struct ItemInfoSpriteSheet {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
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

        Self { texture, layout }
    }
}
