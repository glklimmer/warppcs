use bevy::{prelude::*, sprite::Anchor};
use bevy_replicon::prelude::*;

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
#[require(
    Replicated,
    Transform,
    BoxCollider,
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
    BuildStatus(|| BuildStatus::Built)
)]
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

    pub fn collider(&self) -> BoxCollider {
        match self {
            Building::MainBuilding { level } => match level {
                MainBuildingLevels::Tent => BoxCollider {
                    dimension: Vec2::new(44., 35.),
                    offset: Some(Vec2::new(0., 17.5)),
                },
                MainBuildingLevels::Hall => BoxCollider {
                    dimension: Vec2::new(64., 48.),
                    offset: Some(Vec2::new(0., 24.)),
                },
                MainBuildingLevels::Castle => BoxCollider {
                    dimension: Vec2::new(64., 48.),
                    offset: Some(Vec2::new(0., 24.)),
                },
            },
            Building::Archer => BoxCollider {
                dimension: Vec2::new(100., 50.),
                offset: Some(Vec2::new(0., 25.)),
            },
            Building::Warrior => BoxCollider {
                dimension: Vec2::new(80., 40.),
                offset: Some(Vec2::new(0., 20.)),
            },
            Building::Pikeman => BoxCollider {
                dimension: Vec2::new(80., 40.),
                offset: Some(Vec2::new(0., 20.)),
            },
            Building::Wall { level } => match level {
                WallLevels::Basic => BoxCollider {
                    dimension: Vec2::new(20., 11.),
                    offset: Some(Vec2::new(0., 5.5)),
                },
                WallLevels::Wood => BoxCollider {
                    dimension: Vec2::new(23., 36.),
                    offset: Some(Vec2::new(0., 18.)),
                },
                WallLevels::Tower => BoxCollider {
                    dimension: Vec2::new(110., 190.),
                    offset: Some(Vec2::new(0., -45.)),
                },
            },
            Building::Tower => BoxCollider {
                dimension: Vec2::new(200., 100.),
                offset: None,
            },
            Building::GoldFarm => BoxCollider {
                dimension: Vec2::new(80., 40.),
                offset: Some(Vec2::new(0., 20.)),
            },
        }
    }

    pub fn texture(&self, status: BuildStatus) -> &'static str {
        match status {
            BuildStatus::Marker => match self {
                Building::MainBuilding { level: _ } => "sprites/buildings/main_house_blue.png",
                Building::Archer => "sprites/buildings/sign.png",
                Building::Warrior => "sprites/buildings/sign.png",
                Building::Pikeman => "sprites/buildings/sign.png",
                Building::Wall { level: _ } => "sprites/buildings/sign.png",
                Building::Tower => "",
                Building::GoldFarm => "sprites/buildings/sign.png",
            },
            BuildStatus::Built => match self {
                Building::MainBuilding { level } => match level {
                    MainBuildingLevels::Tent => "sprites/buildings/main_house_blue.png",
                    MainBuildingLevels::Hall => "sprites/buildings/main_hall.png",
                    MainBuildingLevels::Castle => "sprites/buildings/main_castle.png",
                },
                Building::Archer => "sprites/buildings/archer_house.png",
                Building::Warrior => "sprites/buildings/warrior_house.png",
                Building::Pikeman => "sprites/buildings/pike_man_house.png",
                Building::Wall { level } => match level {
                    WallLevels::Basic => "sprites/buildings/wall_1.png",
                    WallLevels::Wood => "sprites/buildings/wall_2.png",
                    WallLevels::Tower => "sprites/buildings/wall_3.png",
                },
                Building::Tower => "sprites/buildings/archer_house.png",
                Building::GoldFarm => "sprites/buildings/warrior_house.png",
            },
            BuildStatus::Destroyed => "",
        }
    }
}

const BUILDING_SCALE: Vec3 = Vec3::new(3., 3., 1.);

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
                .with_scale(BUILDING_SCALE),
        }
    }

    pub fn archer(x: f32) -> Self {
        BuildingBundle {
            building: Building::Archer,
            collider: building_collider(&Building::Archer),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75., Layers::Building.as_f32())
                .with_scale(BUILDING_SCALE),
        }
    }

    pub fn warrior(x: f32) -> Self {
        BuildingBundle {
            building: Building::Warrior,
            collider: building_collider(&Building::Warrior),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75., Layers::Building.as_f32())
                .with_scale(BUILDING_SCALE),
        }
    }

    pub fn pikeman(x: f32) -> Self {
        BuildingBundle {
            building: Building::Pikeman,
            collider: building_collider(&Building::Pikeman),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75., Layers::Building.as_f32())
                .with_scale(BUILDING_SCALE),
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
                .with_scale(BUILDING_SCALE),
        }
    }

    pub fn tower() -> Self {
        BuildingBundle {
            building: Building::Tower,
            collider: building_collider(&Building::Tower),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32())
                .with_scale(BUILDING_SCALE),
        }
    }

    pub fn gold_farm(x: f32) -> Self {
        BuildingBundle {
            building: Building::GoldFarm,
            collider: building_collider(&Building::GoldFarm),
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 25., Layers::Building.as_f32())
                .with_scale(BUILDING_SCALE),
        }
    }
}
