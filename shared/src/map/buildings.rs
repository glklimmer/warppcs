use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{scenes::Slot, Layers};
use crate::{physics::collider::BoxCollider, server::buildings::building_collider};

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

pub fn marker(x: f32) -> Slot {
    Slot {
        initial_building: None,
        collider: BoxCollider {
            dimension: Vec2::new(80., 110.),
            offset: Some(Vec2::new(0., -20.)),
        },
        transform: Transform::from_xyz(x, 72., Layers::Building.as_f32())
            .with_scale(BUILDUING_SCALE),
    }
}

pub fn main(x: f32) -> Slot {
    Slot {
        initial_building: Some((
            Building::MainBuilding {
                level: MainBuildingLevels::Tent,
            },
            BuildStatus::Built,
        )),
        collider: BoxCollider {
            dimension: Vec2::new(150., 110.),
            offset: Some(Vec2::new(0., -20.)),
        },
        transform: Transform::from_xyz(x, 72., Layers::Building.as_f32())
            .with_scale(BUILDUING_SCALE),
    }
}

pub fn wall(x: f32) -> Slot {
    Slot {
        initial_building: Some((
            Building::Wall {
                level: WallLevels::Wood,
            },
            BuildStatus::Marker,
        )),
        collider: building_collider(&Building::Wall {
            level: WallLevels::Basic,
        }),
        transform: Transform::from_xyz(x, 145., Layers::Wall.as_f32()).with_scale(BUILDUING_SCALE),
    }
}

pub fn gold_farm(x: f32) -> Slot {
    Slot {
        initial_building: Some((Building::GoldFarm, BuildStatus::Marker)),
        collider: building_collider(&Building::GoldFarm),
        transform: Transform::from_xyz(x, 25., Layers::Building.as_f32())
            .with_scale(BUILDUING_SCALE),
    }
}
