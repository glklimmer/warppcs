use bevy::{prelude::*, sprite::Anchor};

use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, PlayerColor, enum_map::*, networking::UnitType, server::entities::health::Health,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Mappable)]
pub enum MainBuildingLevels {
    Tent,
    Hall,
    Castle,
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Mappable)]
pub enum BuildStatus {
    Marker,
    Constructing,
    Built { indicator: HealthIndicator },
    Destroyed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Mappable)]
pub enum HealthIndicator {
    Healthy,
    Light,
    Medium,
    Heavy,
}

pub struct Cost {
    pub gold: u16,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
#[require(
    Replicated,
    Transform,
    BoxCollider = marker_collider(),
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    BuildStatus = BuildStatus::Marker,
)]
pub struct RecruitBuilding;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = BoxCollider{
        dimension: Vec2::new(28., 26.),
        offset: Some(Vec2::new(0., 13.)),
    },
)]
pub struct RespawnZone {
    respawn_timer: Timer,
}

impl Default for RespawnZone {
    fn default() -> Self {
        Self {
            respawn_timer: Timer::from_seconds(2., TimerMode::Once),
        }
    }
}

impl RespawnZone {
    pub fn respawn_timer_finished(&self) -> bool {
        self.respawn_timer.finished()
    }

    pub fn respawn_timer_reset(&mut self) {
        self.respawn_timer.reset();
    }
}

pub fn respawn_timer(mut recruit_buildings: Query<&mut RespawnZone>, time: Res<Time>) {
    for mut recruit_building in &mut recruit_buildings.iter_mut() {
        recruit_building.respawn_timer.tick(time.delta());
    }
}

#[derive(Component, Debug, Copy, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    BoxCollider = marker_collider(),
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    BuildStatus = BuildStatus::Marker,
)]
pub struct Building {
    pub building_type: BuildingType,
    pub color: PlayerColor,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Mappable)]
pub enum BuildingType {
    MainBuilding { level: MainBuildingLevels },
    Unit { weapon: UnitType },
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Mappable)]
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
        self.building_type.upgrade().map(|bt| Self {
            building_type: bt,
            color: self.color,
        })
    }

    pub fn can_upgrade(&self) -> bool {
        self.building_type.upgrade().is_some()
    }

    pub fn collider(&self) -> BoxCollider {
        self.building_type.collider()
    }

    pub fn marker_texture() -> &'static str {
        "sprites/buildings/sign.png"
    }

    pub fn health(&self) -> Health {
        self.building_type.health()
    }

    pub fn cost(&self) -> Cost {
        self.building_type.cost()
    }

    pub fn is_recruit_building(&self) -> bool {
        self.building_type.is_recruit_building()
    }

    pub fn unit_type(&self) -> Option<UnitType> {
        self.building_type.unit_type()
    }

    pub fn time(&self) -> f32 {
        self.building_type.time()
    }
}

impl BuildingType {
    pub fn upgrade(&self) -> Option<Self> {
        match *self {
            BuildingType::MainBuilding { level } => level
                .next_level()
                .map(|level| BuildingType::MainBuilding { level }),
            BuildingType::Wall { level } => {
                level.next_level().map(|level| BuildingType::Wall { level })
            }
            BuildingType::Unit { weapon: _ } => None,
            BuildingType::Tower => None,
            BuildingType::GoldFarm => None,
        }
    }

    pub fn can_upgrade(&self) -> bool {
        self.upgrade().is_some()
    }

    pub fn collider(&self) -> BoxCollider {
        match self {
            BuildingType::MainBuilding { level } => match level {
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
            BuildingType::Unit { weapon } => match weapon {
                UnitType::Shieldwarrior => BoxCollider {
                    dimension: Vec2::new(80., 40.),
                    offset: Some(Vec2::new(0., 20.)),
                },
                UnitType::Pikeman => BoxCollider {
                    dimension: Vec2::new(80., 40.),
                    offset: Some(Vec2::new(0., 20.)),
                },
                UnitType::Archer => BoxCollider {
                    dimension: Vec2::new(100., 50.),
                    offset: Some(Vec2::new(0., 25.)),
                },
                UnitType::Bandit => todo!(),
                UnitType::Commander => todo!(),
            },
            BuildingType::Wall { level } => match level {
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
            BuildingType::Tower => BoxCollider {
                dimension: Vec2::new(200., 100.),
                offset: None,
            },
            BuildingType::GoldFarm => BoxCollider {
                dimension: Vec2::new(80., 40.),
                offset: Some(Vec2::new(0., 20.)),
            },
        }
    }

    pub fn health(&self) -> Health {
        let hitpoints = match self {
            BuildingType::MainBuilding { level } => match level {
                MainBuildingLevels::Tent => 1200.,
                MainBuildingLevels::Hall => 3600.,
                MainBuildingLevels::Castle => 6400.,
            },
            BuildingType::Unit { weapon: _ } => 800.,
            BuildingType::Wall { level } => match level {
                WallLevels::Basic => 600.,
                WallLevels::Wood => 1200.,
                WallLevels::Tower => 2400.,
            },
            BuildingType::Tower => 400.,
            BuildingType::GoldFarm => 600.,
        };
        Health { hitpoints }
    }

    pub fn cost(&self) -> Cost {
        let gold = match self {
            BuildingType::MainBuilding { level } => match level {
                MainBuildingLevels::Tent => 0,
                MainBuildingLevels::Hall => 1000,
                MainBuildingLevels::Castle => 4000,
            },
            BuildingType::Unit { weapon: _ } => 200,
            BuildingType::Wall { level } => match level {
                WallLevels::Basic => 100,
                WallLevels::Wood => 300,
                WallLevels::Tower => 900,
            },
            BuildingType::Tower => 150,
            BuildingType::GoldFarm => 200,
        };
        Cost { gold }
    }

    pub fn is_recruit_building(&self) -> bool {
        match self {
            BuildingType::MainBuilding { level: _ } => true,
            BuildingType::Unit { weapon: _ } => true,
            BuildingType::Wall { level: _ } => false,
            BuildingType::Tower => false,
            BuildingType::GoldFarm => false,
        }
    }

    pub fn unit_type(&self) -> Option<UnitType> {
        match *self {
            BuildingType::MainBuilding { level: _ } => Some(UnitType::Commander),
            BuildingType::Unit { weapon: unit_type } => Some(unit_type),
            BuildingType::Wall { level: _ } | BuildingType::Tower | BuildingType::GoldFarm => None,
        }
    }

    fn time(&self) -> f32 {
        match self {
            BuildingType::MainBuilding { level } => match level {
                MainBuildingLevels::Tent => 10.,
                MainBuildingLevels::Hall => 50.,
                MainBuildingLevels::Castle => 200.,
            },
            BuildingType::Unit { weapon: _ } => 6.,
            BuildingType::Wall { level } => match level {
                WallLevels::Basic => 5.,
                WallLevels::Wood => 20.,
                WallLevels::Tower => 50.,
            },
            BuildingType::Tower => todo!(),
            BuildingType::GoldFarm => 5.,
        }
    }
}

fn marker_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(28., 26.),
        offset: Some(Vec2::new(0., 13.)),
    }
}
