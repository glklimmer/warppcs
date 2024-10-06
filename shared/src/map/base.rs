use bevy::prelude::*;

use super::{Layers, TriggerZone};
use crate::BoxCollider;

#[derive(Component)]
pub struct MainBuilding;

#[derive(Component)]
pub enum MainBuildingLevel {
    First,
    Second,
    Third,
}

#[derive(Component)]
pub enum BuildStatus {
    None,
    Built,
}

#[derive(Component)]
pub enum Building {
    Archer,
    Warrior,
    Pikeman,
}

#[derive(Component)]
pub enum UpgradableBuilding {
    Wall,
    Tower,
}

#[derive(Component)]
pub enum Upgradable {
    None,
    First,
    Second,
    Third,
}

#[derive(Bundle)]
pub struct MainBuildingBundle {
    pub base: MainBuilding,
    pub collider: BoxCollider,
    pub main_building_level: MainBuildingLevel,
    pub trigger_zone: TriggerZone,
    pub transform: Transform,
}

impl MainBuildingBundle {
    pub fn new() -> Self {
        MainBuildingBundle {
            base: MainBuilding,
            collider: BoxCollider(Vec2::new(200., 100.)),
            main_building_level: MainBuildingLevel::First,
            trigger_zone: TriggerZone,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32()),
        }
    }
}

impl Default for MainBuildingBundle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Bundle)]
pub struct BuildingBundle {
    pub building: Building,
    pub collider: BoxCollider,
    pub build_status: BuildStatus,
    pub trigger_zone: TriggerZone,
    pub transform: Transform,
}

impl BuildingBundle {
    pub fn archer() -> Self {
        BuildingBundle {
            building: Building::Archer,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::None,
            trigger_zone: TriggerZone,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32()),
        }
    }

    pub fn warrior() -> Self {
        BuildingBundle {
            building: Building::Warrior,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::None,
            trigger_zone: TriggerZone,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32()),
        }
    }

    pub fn pikeman() -> Self {
        BuildingBundle {
            building: Building::Pikeman,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::None,
            trigger_zone: TriggerZone,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32()),
        }
    }
}

#[derive(Bundle)]
pub struct UpgradableBuildingBundle {
    pub building: UpgradableBuilding,
    pub upgrade: Upgradable,
    pub collider: BoxCollider,
    pub build_status: BuildStatus,
    pub trigger_zone: TriggerZone,
    pub transform: Transform,
}

impl UpgradableBuildingBundle {
    pub fn wall() -> Self {
        UpgradableBuildingBundle {
            building: UpgradableBuilding::Wall,
            upgrade: Upgradable::None,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::None,
            trigger_zone: TriggerZone,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32()),
        }
    }

    pub fn tower() -> Self {
        UpgradableBuildingBundle {
            building: UpgradableBuilding::Tower,
            upgrade: Upgradable::None,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::None,
            trigger_zone: TriggerZone,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32()),
        }
    }
}

pub struct BaseScene {
    pub main_building: MainBuildingBundle,
    pub archer_building: BuildingBundle,
    pub warrior_building: BuildingBundle,
    pub pikeman_building: BuildingBundle,
    pub left_wall: UpgradableBuildingBundle,
    pub right_wall: UpgradableBuildingBundle,
}

impl BaseScene {
    pub fn new() -> Self {
        BaseScene {
            main_building: MainBuildingBundle::new(),
            archer_building: BuildingBundle::archer(),
            warrior_building: BuildingBundle::warrior(),
            pikeman_building: BuildingBundle::pikeman(),
            left_wall: UpgradableBuildingBundle::wall(),
            right_wall: UpgradableBuildingBundle::tower(),
        }
    }
}

impl Default for BaseScene {
    fn default() -> Self {
        Self::new()
    }
}
