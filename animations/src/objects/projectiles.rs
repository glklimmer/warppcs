use bevy::prelude::*;
use shared::enum_map::*;

use crate::StaticSpriteSheet;

#[derive(Debug, Clone, Copy, Mappable)]
pub enum Projectiles {
    Arrow,
}

#[derive(Resource)]
pub struct ProjectileSpriteSheet {
    pub sprite_sheet: StaticSpriteSheet<Projectiles>,
}

impl FromWorld for ProjectileSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/humans/HumansProjectiles.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2 { x: 16, y: 16 },
            5,
            3,
            None,
            None,
        ));

        let parts = EnumMap::new(|c| match c {
            Projectiles::Arrow => 1,
        });

        Self {
            sprite_sheet: StaticSpriteSheet::new(world, texture, layout, parts),
        }
    }
}
