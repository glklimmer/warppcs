use bevy::prelude::*;

use super::{
    super::super::enum_map::*, GameScene, GameSceneId, GameSceneType, SceneSlot, SceneSlotIndicator,
};
use crate::{
    entities::{
        chest::{chest, ChestBundle},
        spawn_point::spawn_point,
    },
    map::{
        buildings::{gold_farm, main, marker, wall, BuildingBundle, ItemSlotBundle},
        spawn_point::SpawnPointBundle,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct BaseScene {
    pub main_building: BuildingBundle,
    pub starter_chest: ChestBundle,
    pub first_right_slot: ItemSlotBundle,
    pub first_left_slot: ItemSlotBundle,
    pub second_right_slot: ItemSlotBundle,
    pub left_wall: BuildingBundle,
    pub right_wall: BuildingBundle,
    pub left_gold_farm: BuildingBundle,
    pub right_gold_farm: BuildingBundle,
    pub left_spawn_point: SpawnPointBundle,
    pub right_spawn_point: SpawnPointBundle,
}

#[derive(Copy, Clone, Component, Serialize, Deserialize, Debug, PartialEq, Eq, Mappable)]
pub enum BaseSceneIndicator {
    MainBuilding,
    StarterChest,
    FirstRightSlot,
    FirstLeftSlot,
    SecondRightSlot,
    LeftWall,
    RightWall,
    LeftGoldFarm,
    RightGoldFarm,
    LeftSpawnPoint,
    RightSpawnPoint,
}

fn define_base_scene(id: GameSceneId) -> GameScene {
    GameScene {
        id,
        game_scene_type: GameSceneType::Base,
        slots: vec![
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::MainBuilding),
                slot: main(0.),
            },
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::StarterChest),
                slot: chest(200.),
            },
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::FirstRightSlot),
                slot: marker(400.),
            },
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::FirstLeftSlot),
                slot: marker(-400.),
            },
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::SecondRightSlot),
                slot: marker(650.),
            },
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::LeftWall),
                slot: wall(-1050.),
            },
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::RightWall),
                slot: wall(1050.),
            },
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::LeftGoldFarm),
                slot: gold_farm(-800.),
            },
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::RightGoldFarm),
                slot: gold_farm(875.),
            },
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::LeftSpawnPoint),
                slot: spawn_point(-1800.),
            },
            SceneSlot {
                indicator: SceneSlotIndicator::Base(BaseSceneIndicator::RightSpawnPoint),
                slot: spawn_point(1800.),
            },
        ],
        left_game_scenes: Vec::new(),
        right_game_scenes: Vec::new(),
    }
}

impl BaseScene {
    pub fn new() -> Self {
        BaseScene {
            main_building: BuildingBundle::main(0.),
            starter_chest: ChestBundle::new(200.),
            first_right_slot: ItemSlotBundle::new(400.),
            first_left_slot: ItemSlotBundle::new(-400.),
            second_right_slot: ItemSlotBundle::new(650.),
            left_wall: BuildingBundle::wall(-1050.),
            right_wall: BuildingBundle::wall(1050.),
            left_gold_farm: BuildingBundle::gold_farm(-800.),
            right_gold_farm: BuildingBundle::gold_farm(875.),
            left_spawn_point: SpawnPointBundle::new(-1800.),
            right_spawn_point: SpawnPointBundle::new(1800.),
        }
    }
}

impl Default for BaseScene {
    fn default() -> Self {
        Self::new()
    }
}
