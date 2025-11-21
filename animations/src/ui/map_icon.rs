use bevy::prelude::*;

use shared::enum_map::*;

use crate::StaticSpriteSheet;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum MapIcons {
    #[default]
    Player,
    Bandit,
    Mystery,
}

#[derive(Resource)]
pub struct MapIconSpriteSheet {
    pub sprite_sheet: StaticSpriteSheet<MapIcons>,
}

impl FromWorld for MapIconSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/ui/map_icons.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2 { x: 46, y: 36 },
            3,
            1,
            None,
            None,
        ));

        let parts = EnumMap::new(|c| match c {
            MapIcons::Player => 0,
            MapIcons::Bandit => 1,
            MapIcons::Mystery => 2,
        });

        Self {
            sprite_sheet: StaticSpriteSheet::new(world, texture, layout, parts),
        }
    }
}
