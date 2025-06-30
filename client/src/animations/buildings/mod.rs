use bevy::prelude::*;

use main::tent;
use shared::{
    PlayerColor,
    enum_map::*,
    map::buildings::{BuildStatus, Building, MainBuildingLevels},
};

use crate::{
    animations::{AnimationSound, AnimationSoundTrigger},
    entities::highlight::Highlighted,
    sound::CRAFTING_SOUND_PATH,
};

use super::{AnimationSpriteSheet, sprite_variant_loader::SpriteVariants};

mod main;

#[derive(Resource)]
pub struct BuildingSpriteSheets {
    pub sprite_sheets: EnumMap<Building, AnimationSpriteSheet<BuildStatus, SpriteVariants>>,
}

impl FromWorld for BuildingSpriteSheets {
    fn from_world(world: &mut World) -> Self {
        let tent = tent(world);

        let sprite_sheets = EnumMap::new(|c| match c {
            Building::MainBuilding { level } => match level {
                MainBuildingLevels::Tent => tent.clone(),
                MainBuildingLevels::Hall => tent.clone(),
                MainBuildingLevels::Castle => tent.clone(),
            },
            Building::Unit { weapon: _ } => tent.clone(),
            Building::Wall { level: _ } => tent.clone(),
            Building::Tower => tent.clone(),
            Building::GoldFarm => tent.clone(),
        });

        BuildingSpriteSheets { sprite_sheets }
    }
}

pub fn update_building_sprite(
    mut buildings: Query<
        (
            Entity,
            &mut Sprite,
            &Building,
            &BuildStatus,
            &PlayerColor,
            Option<&mut Highlighted>,
        ),
        Or<(Changed<Building>, Changed<BuildStatus>)>,
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    building_sprite_sheet: Res<BuildingSpriteSheets>,
    variants: Res<Assets<SpriteVariants>>,
) {
    for (entity, mut sprite, building, status, color, maybe_highlight) in buildings.iter_mut() {
        let sprite_sheet = building_sprite_sheet.sprite_sheets.get(*building);
        let handle = &sprite_sheet.texture;
        let sprite_variants = variants.get(handle).unwrap();
        let animation = sprite_sheet.animations.get(*status);

        sprite.texture_atlas = Some(TextureAtlas {
            layout: sprite_sheet.layout.clone(),
            index: animation.first_sprite_index,
        });
        sprite.image = sprite_variants.variants.get(*color).clone();

        // TODO: add animations
        // commands.entity(trigger.target()).insert(animation.clone());

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = animation.first_sprite_index;
        }

        if let Some(mut highlight) = maybe_highlight {
            highlight.original_handle = asset_server.load(building.texture(*status));
        }

        if status.eq(&BuildStatus::Built) {
            commands.entity(entity).insert(AnimationSound {
                sound_handles: vec![
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/hammer_1.ogg"
                    )),
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/hammer_2.ogg"
                    )),
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/sawing_wood_1.ogg"
                    )),
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/sawing_wood_2.ogg"
                    )),
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/sawing_wood_3.ogg"
                    )),
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/hammering_&_chiseling_stone_1.ogg"
                    )),
                ],
                sound_trigger: AnimationSoundTrigger::OnEnter,
            });
        }
    }
}
