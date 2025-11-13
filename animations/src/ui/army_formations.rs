use bevy::prelude::*;

use shared::enum_map::*;

use crate::StaticSpriteSheet;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum FormationIcons {
    #[default]
    Front,
    Middle,
    Back,
}

#[derive(Resource)]
pub struct FormationIconSpriteSheet {
    pub sprite_sheet: StaticSpriteSheet<FormationIcons>,
}

impl FromWorld for FormationIconSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("ui/commander/formation.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2 { x: 50, y: 50 },
            3,
            1,
            None,
            None,
        ));

        let parts = EnumMap::new(|c| match c {
            FormationIcons::Front => 2,
            FormationIcons::Middle => 1,
            FormationIcons::Back => 0,
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
