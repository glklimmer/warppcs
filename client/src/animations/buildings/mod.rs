use bevy::prelude::*;

use archer::archer_building;
use gold_farm::gold_farm_building;
use pikeman::pikeman_building;
use shared::{
    enum_map::*,
    map::buildings::{BuildStatus, Building, BuildingType, MainBuildingLevels, WallLevels},
    networking::UnitType,
};
use shieldwarrior::shieldwarrior_building;
use tent::tent_building;
use wall_basic::wall_basic_building;
use wall_tower::wall_tower_building;
use wall_wood::wall_wood_building;

use crate::{
    animations::{AnimationSound, AnimationSoundTrigger},
    entities::highlight::Highlighted,
    sound::CRAFTING_SOUND_PATH,
};

use super::{AnimationSpriteSheet, sprite_variant_loader::SpriteVariants};

mod archer;
mod gold_farm;
mod pikeman;
mod shieldwarrior;
mod tent;
mod wall_basic;
mod wall_tower;
mod wall_wood;

#[derive(Resource)]
pub struct BuildingSpriteSheets {
    pub sprite_sheets: EnumMap<BuildingType, AnimationSpriteSheet<BuildStatus, SpriteVariants>>,
}

impl FromWorld for BuildingSpriteSheets {
    fn from_world(world: &mut World) -> Self {
        let tent = tent_building(world);
        let archer = archer_building(world);
        let shieldwarrior = shieldwarrior_building(world);
        let pikeman = pikeman_building(world);
        let wall_basic = wall_basic_building(world);
        let wall_wood = wall_wood_building(world);
        let wall_tower = wall_tower_building(world);
        let gold_farm = gold_farm_building(world);

        let sprite_sheets = EnumMap::new(|c| match c {
            BuildingType::MainBuilding { level } => match level {
                MainBuildingLevels::Tent => tent.clone(),
                MainBuildingLevels::Hall => tent.clone(),
                MainBuildingLevels::Castle => tent.clone(),
            },
            BuildingType::Unit { weapon } => match weapon {
                UnitType::Archer => archer.clone(),
                UnitType::Shieldwarrior => shieldwarrior.clone(),
                UnitType::Pikeman => pikeman.clone(),
                UnitType::Bandit => tent.clone(),
                UnitType::Commander => tent.clone(),
            },
            BuildingType::Wall { level } => match level {
                WallLevels::Basic => wall_basic.clone(),
                WallLevels::Wood => wall_wood.clone(),
                WallLevels::Tower => wall_tower.clone(),
            },
            BuildingType::Tower => tent.clone(),
            BuildingType::GoldFarm => gold_farm.clone(),
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
            Option<&mut Highlighted>,
        ),
        Or<(Changed<Building>, Changed<BuildStatus>)>,
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    building_sprite_sheet: Res<BuildingSpriteSheets>,
    variants: Res<Assets<SpriteVariants>>,
) {
    for (entity, mut sprite, building, status, maybe_highlight) in buildings.iter_mut() {
        let sprite_sheet = building_sprite_sheet
            .sprite_sheets
            .get(building.building_type);
        let handle = &sprite_sheet.texture;
        let sprite_variants = variants.get(handle).unwrap();
        let animation = sprite_sheet.animations.get(*status).clone();

        sprite.texture_atlas = Some(TextureAtlas {
            layout: sprite_sheet.layout.clone(),
            index: animation.first_sprite_index,
        });
        let handle = sprite_variants.variants.get(building.color).clone();
        sprite.image = handle.clone();

        let animation = animation.with_total_duration(building.time());
        commands.entity(entity).insert(animation.clone());

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = animation.first_sprite_index;
        }

        if let Some(mut highlight) = maybe_highlight {
            highlight.original_handle = handle.clone();
        }

        if let BuildStatus::Built { indicator: _ } = status {
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
