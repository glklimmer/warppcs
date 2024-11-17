use bevy::prelude::*;

use super::{spawn_point::SpawnPointBundle, Layers};
use crate::BoxCollider;

#[derive(Component, Copy, Clone)]
pub struct MainBuilding;

#[derive(Component, Copy, Clone)]
pub enum MainBuildingLevel {
    First,
    Second,
    Third,
}

#[derive(Component, Copy, Clone, PartialEq, Eq)]
pub enum BuildStatus {
    None,
    Built,
}

#[derive(Component, Copy, Clone)]
pub enum Building {
    Archer,
    Warrior,
    Pikeman,
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum UpgradableBuilding {
    Wall,
    Tower,
    GoldFarm,
}

#[derive(Component, Copy, Clone)]
pub enum Upgradable {
    First,
    Second,
    Third,
}

#[derive(Bundle, Copy, Clone)]
pub struct MainBuildingBundle {
    pub base: MainBuilding,
    pub collider: BoxCollider,
    pub main_building_level: MainBuildingLevel,
    pub transform: Transform,
}

impl MainBuildingBundle {
    pub fn new() -> Self {
        MainBuildingBundle {
            base: MainBuilding,
            collider: BoxCollider(Vec2::new(200., 100.)),
            main_building_level: MainBuildingLevel::First,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32()),
        }
    }
}

impl Default for MainBuildingBundle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Bundle, Copy, Clone)]
pub struct BuildingBundle {
    pub building: Building,
    pub collider: BoxCollider,
    pub build_status: BuildStatus,
    pub transform: Transform,
}

impl BuildingBundle {
    pub fn archer() -> Self {
        BuildingBundle {
            building: Building::Archer,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::None,
            transform: Transform::from_xyz(400., 50., Layers::Building.as_f32()),
        }
    }

    pub fn warrior() -> Self {
        BuildingBundle {
            building: Building::Warrior,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::None,
            transform: Transform::from_xyz(-400., 50., Layers::Building.as_f32()),
        }
    }

    pub fn pikeman() -> Self {
        BuildingBundle {
            building: Building::Pikeman,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::None,
            transform: Transform::from_xyz(650., 50., Layers::Building.as_f32()),
        }
    }
}

#[derive(Bundle, Copy, Clone)]
pub struct UpgradableBuildingBundle {
    pub building: UpgradableBuilding,
    pub collider: BoxCollider,
    pub build_status: BuildStatus,
    pub transform: Transform,
}

impl UpgradableBuildingBundle {
    pub fn wall(x: f32) -> Self {
        UpgradableBuildingBundle {
            building: UpgradableBuilding::Wall,
            collider: BoxCollider(Vec2::new(50., 75.)),
            build_status: BuildStatus::None,
            transform: Transform::from_xyz(x, 75. / 2., Layers::Building.as_f32()),
        }
    }

    pub fn tower() -> Self {
        UpgradableBuildingBundle {
            building: UpgradableBuilding::Tower,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::None,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32()),
        }
    }

    pub fn gold_farm(x: f32) -> Self {
        UpgradableBuildingBundle {
            building: UpgradableBuilding::GoldFarm,
            collider: BoxCollider(Vec2::new(200., 50.)),
            build_status: BuildStatus::None,
            transform: Transform::from_xyz(x, 25., Layers::Building.as_f32()),
        }
    }
}

#[derive(Copy, Clone)]
pub struct BaseScene {
    pub main_building: MainBuildingBundle,
    pub archer_building: BuildingBundle,
    pub warrior_building: BuildingBundle,
    pub pikeman_building: BuildingBundle,
    pub left_wall: UpgradableBuildingBundle,
    pub right_wall: UpgradableBuildingBundle,
    pub left_gold_farm: UpgradableBuildingBundle,
    pub right_gold_farm: UpgradableBuildingBundle,
    pub left_spawn_point: SpawnPointBundle,
    pub right_spawn_point: SpawnPointBundle,
}

impl BaseScene {
    pub fn new() -> Self {
        BaseScene {
            main_building: MainBuildingBundle::new(),
            archer_building: BuildingBundle::archer(),
            warrior_building: BuildingBundle::warrior(),
            pikeman_building: BuildingBundle::pikeman(),
            left_wall: UpgradableBuildingBundle::wall(-800.),
            right_wall: UpgradableBuildingBundle::wall(1050.),
            left_gold_farm: UpgradableBuildingBundle::gold_farm(-1450.),
            right_gold_farm: UpgradableBuildingBundle::gold_farm(1450.),
            left_spawn_point: SpawnPointBundle::new(-1200.),
            right_spawn_point: SpawnPointBundle::new(1200.),
        }
    }
}

impl Default for BaseScene {
    fn default() -> Self {
        Self::new()
    }
}
