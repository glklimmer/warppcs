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
    Marker,
    Built,
}

#[derive(Component, Copy, Clone)]
pub struct Cost {
    pub gold: u16,
}

#[derive(Component, Copy, Clone, PartialEq, Eq)]
pub enum Building {
    Archer,
    Warrior,
    Pikeman,
    Wall,
    Tower,
    GoldFarm,
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
    pub cost: Cost,
}

impl BuildingBundle {
    pub fn archer() -> Self {
        BuildingBundle {
            building: Building::Archer,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(400., 50., Layers::Building.as_f32()),
            cost: Cost { gold: 200 },
        }
    }

    pub fn warrior() -> Self {
        BuildingBundle {
            building: Building::Warrior,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(-400., 50., Layers::Building.as_f32()),
            cost: Cost { gold: 200 },
        }
    }

    pub fn pikeman() -> Self {
        BuildingBundle {
            building: Building::Pikeman,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(650., 50., Layers::Building.as_f32()),
            cost: Cost { gold: 200 },
        }
    }

    pub fn wall(x: f32) -> Self {
        BuildingBundle {
            building: Building::Wall,
            collider: BoxCollider(Vec2::new(50., 75.)),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75. / 2., Layers::Building.as_f32()),
            cost: Cost { gold: 100 },
        }
    }

    pub fn tower() -> Self {
        BuildingBundle {
            building: Building::Tower,
            collider: BoxCollider(Vec2::new(200., 100.)),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32()),
            cost: Cost { gold: 150 },
        }
    }

    pub fn gold_farm(x: f32) -> Self {
        BuildingBundle {
            building: Building::GoldFarm,
            collider: BoxCollider(Vec2::new(200., 50.)),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 25., Layers::Building.as_f32()),
            cost: Cost { gold: 50 },
        }
    }
}

#[derive(Copy, Clone)]
pub struct BaseScene {
    pub main_building: MainBuildingBundle,
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

impl BaseScene {
    pub fn new() -> Self {
        BaseScene {
            main_building: MainBuildingBundle::new(),
            archer_building: BuildingBundle::archer(),
            warrior_building: BuildingBundle::warrior(),
            pikeman_building: BuildingBundle::pikeman(),
            left_wall: BuildingBundle::wall(-800.),
            right_wall: BuildingBundle::wall(1050.),
            left_gold_farm: BuildingBundle::gold_farm(-1450.),
            right_gold_farm: BuildingBundle::gold_farm(1450.),
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
