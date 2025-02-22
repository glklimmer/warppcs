use bevy::prelude::*;

use crate::map::{buildings::BuildingBundle, spawn_point::SpawnPointBundle};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct BaseScene {
    pub main_building: BuildingBundle,
    pub archer_building: BuildingBundle,
    pub warrior_building: BuildingBundle,
    pub pikeman_building: BuildingBundle,
    pub left_wall: BuildingBundle,
    pub right_wall: BuildingBundle,
    pub left_gold_farm: BuildingBundle,
    pub right_gold_farm: BuildingBundle,
    pub left_spawn_point: SpawnPointBundle,
    pub right_spawn_point: SpawnPointBundle,
}

#[derive(Copy, Clone, Component, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum BaseSceneIndicator {
    MainBuilding,
    ArcherBuilding,
    WarriorBuilding,
    PikemanBuilding,
    LeftWall,
    RightWall,
    LeftGoldFarm,
    RightGoldFarm,
    LeftSpawnPoint,
    RightSpawnPoint,
}

impl BaseScene {
    pub fn new() -> Self {
        BaseScene {
            main_building: BuildingBundle::main(0.),
            archer_building: BuildingBundle::archer(400.),
            warrior_building: BuildingBundle::warrior(-400.),
            pikeman_building: BuildingBundle::pikeman(650.),
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
