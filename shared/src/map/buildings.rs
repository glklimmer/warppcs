use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::Layers;
use crate::{server::buildings::building_collider, BoxCollider};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MainBuildingLevels {
    Tent,
    Hall,
    Castle,
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildStatus {
    Marker,
    Built,
    Destroyed,
}

#[derive(Component, Debug, Copy, Clone)]
pub struct RecruitBuilding;

pub struct Cost {
    pub gold: u16,
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Building {
    MainBuilding { level: MainBuildingLevels },
    Archer,
    Warrior,
    Pikeman,
    Wall { level: WallLevels },
    Tower,
    GoldFarm,
}

impl MainBuildingLevels {
    fn next_level(&self) -> Option<MainBuildingLevels> {
        match self {
            MainBuildingLevels::Tent => Some(MainBuildingLevels::Hall),
            MainBuildingLevels::Hall => Some(MainBuildingLevels::Castle),
            MainBuildingLevels::Castle => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WallLevels {
    Basic,
    Wood,
    Tower,
}

impl WallLevels {
    fn next_level(&self) -> Option<WallLevels> {
        match self {
            WallLevels::Basic => Some(WallLevels::Wood),
            WallLevels::Wood => Some(WallLevels::Tower),
            WallLevels::Tower => None,
        }
    }
}

impl Building {
    pub fn upgrade_building(&self) -> Option<Self> {
        match *self {
            Building::MainBuilding { level } => level
                .next_level()
                .map(|level| (Building::MainBuilding { level })),
            Building::Wall { level } => level.next_level().map(|level| (Building::Wall { level })),
            Building::Archer => None,
            Building::Warrior => None,
            Building::Pikeman => None,
            Building::Tower => None,
            Building::GoldFarm => None,
        }
    }

    pub fn can_upgrade(&self) -> bool {
        self.upgrade_building().is_some()
    }
}

const BUILDUING_SCALE: Vec3 = Vec3::new(3., 3., 1.);

#[derive(Bundle, Clone, Copy)]
pub struct BuildingBundle {
    pub building: Building,
    pub collider: BoxCollider,
    pub build_status: BuildStatus,
    pub transform: Transform,
}

impl BuildingBundle {
    pub fn main(x: f32) -> Self {
        Self {
            building: Building::MainBuilding {
                level: MainBuildingLevels::Tent,
            },
            collider: BoxCollider {
                dimension: Vec2::new(150., 110.),
                offset: Some(Vec2::new(0., -20.)),
            },
            build_status: BuildStatus::Built,
            transform: Transform::from_xyz(x, 72., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
        }
    }

    pub fn archer(x: f32) -> Self {
        BuildingBundle {
            building: Building::Archer,
            collider: building_collider(&Building::Archer),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
        }
    }

    pub fn warrior(x: f32) -> Self {
        BuildingBundle {
            building: Building::Warrior,
            collider: building_collider(&Building::Warrior),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
        }
    }

    pub fn pikeman(x: f32) -> Self {
        BuildingBundle {
            building: Building::Pikeman,
            collider: building_collider(&Building::Pikeman),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
        }
    }

    pub fn wall(x: f32) -> Self {
        BuildingBundle {
            building: Building::Wall {
                level: WallLevels::Basic,
            },
            collider: building_collider(&Building::Wall {
                level: WallLevels::Basic,
            }),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 145., Layers::Wall.as_f32())
                .with_scale(BUILDUING_SCALE),
        }
    }

    pub fn tower() -> Self {
        BuildingBundle {
            building: Building::Tower,
            collider: building_collider(&Building::Tower),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
        }
    }

    pub fn gold_farm(x: f32) -> Self {
        BuildingBundle {
            building: Building::GoldFarm,
            collider: building_collider(&Building::GoldFarm),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 25., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
        }
    }
}
