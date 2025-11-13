use bevy::prelude::*;
use shared::enum_map::*;

use crate::StaticSpriteSheet;

#[derive(Debug, Clone, Copy, Mappable)]
pub enum CommanderMenuNodes {
    Flag,
    Camp,
    Formation,
}

#[derive(Resource)]
pub struct CommanderMenuSpriteSheet {
    pub sprite_sheet: StaticSpriteSheet<CommanderMenuNodes>,
}

impl FromWorld for CommanderMenuSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("ui/commander/menu-nodes.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2 { x: 50, y: 50 },
            3,
            1,
            None,
            None,
        ));

        let parts = EnumMap::new(|c| match c {
            CommanderMenuNodes::Flag => 2,
            CommanderMenuNodes::Camp => 0,
            CommanderMenuNodes::Formation => 1,
        });

        Self {
            sprite_sheet: StaticSpriteSheet::new(world, texture, layout, parts),
        }
    }
}
