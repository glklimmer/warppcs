use bevy::prelude::*;

use crate::map::{
    buildings::{BuildingBundle, BuildingMarkerBundle},
    spawn_point::SpawnPointBundle,
    ChestBundle,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct BaseScene {
    pub main_building: BuildingBundle,
    pub starter_chest: ChestBundle,
    pub first_right_marker: BuildingMarkerBundle,
    pub first_left_marker: BuildingMarkerBundle,
    pub second_right_marker: BuildingMarkerBundle,
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
    StarterChest,
    FirstRightMarker,
    FirstLeftMarker,
    SecondRightMarker,
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
            starter_chest: ChestBundle::new(200.),
            first_right_marker: BuildingMarkerBundle::new(400.),
            first_left_marker: BuildingMarkerBundle::new(-400.),
            second_right_marker: BuildingMarkerBundle::new(650.),
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
