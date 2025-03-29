use bevy::{prelude::*, sprite::Anchor};
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{server::entities::health::Health, BoxCollider};

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
    BoxCollider(marker_collider),
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
    BuildStatus(|| BuildStatus::Marker),
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

    pub fn health(&self) -> Health {
        let hitpoints = match self {
            Building::MainBuilding { level } => match level {
                MainBuildingLevels::Tent => 1200.,
                MainBuildingLevels::Hall => 3600.,
                MainBuildingLevels::Castle => 6400.,
            },
            Building::Archer => 800.,
            Building::Warrior => 800.,
            Building::Pikeman => 800.,
            Building::Wall { level } => match level {
                WallLevels::Basic => 600.,
                WallLevels::Wood => 1200.,
                WallLevels::Tower => 2400.,
            },
            Building::Tower => 400.,
            Building::GoldFarm => 600.,
        };
        Health { hitpoints }
    }

    pub fn cost(&self) -> Cost {
        let gold = match self {
            Building::MainBuilding { level } => match level {
                MainBuildingLevels::Tent => 0,
                MainBuildingLevels::Hall => 1000,
                MainBuildingLevels::Castle => 4000,
            },
            Building::Archer => 200,
            Building::Warrior => 200,
            Building::Pikeman => 200,
            Building::Wall { level } => match level {
                WallLevels::Basic => 100,
                WallLevels::Wood => 300,
                WallLevels::Tower => 900,
            },
            Building::Tower => 150,
            Building::GoldFarm => 200,
        };
        Cost { gold }
    }
}

fn marker_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(28., 26.),
        offset: Some(Vec2::new(0., 13.)),
    }
}
