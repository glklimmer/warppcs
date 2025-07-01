use bevy::{prelude::*, sprite::Anchor};

use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};

use crate::{BoxCollider, enum_map::*, networking::UnitType, server::entities::health::Health};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Mappable)]
pub enum MainBuildingLevels {
    Tent,
    Hall,
    Castle,
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Mappable)]
pub enum BuildStatus {
    Marker,
    Built,
    Destroyed,
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

#[derive(Component, Debug, Copy, Clone, Serialize, Deserialize, Mappable)]
#[require(
    Replicated,
    BoxCollider = marker_collider(),
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    BuildStatus = BuildStatus::Marker,
)]
pub enum Building {
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
        match *self {
            Building::MainBuilding { level } => level
                .next_level()
                .map(|level| (Building::MainBuilding { level })),
            Building::Wall { level } => level.next_level().map(|level| (Building::Wall { level })),
            Building::Unit { weapon: _ } => None,
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
            Building::Unit { weapon } => match weapon {
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

    pub fn marker_texture() -> &'static str {
        "sprites/buildings/sign.png"
    }

    pub fn health(&self) -> Health {
        let hitpoints = match self {
            Building::MainBuilding { level } => match level {
                MainBuildingLevels::Tent => 1200.,
                MainBuildingLevels::Hall => 3600.,
                MainBuildingLevels::Castle => 6400.,
            },
            Building::Unit { weapon: _ } => 800.,
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
            Building::Unit { weapon: _ } => 200,
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

    pub fn is_recruit_building(&self) -> bool {
        match self {
            Building::MainBuilding { level: _ } => true,
            Building::Unit { weapon: _ } => true,
            Building::Wall { level: _ } => false,
            Building::Tower => false,
            Building::GoldFarm => false,
        }
    }

    pub fn unit_type(&self) -> Option<UnitType> {
        match *self {
            Building::MainBuilding { level: _ } => Some(UnitType::Commander),
            Building::Unit { weapon: unit_type } => Some(unit_type),
            Building::Wall { level: _ } | Building::Tower | Building::GoldFarm => None,
        }
    }
}

fn marker_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(28., 26.),
        offset: Some(Vec2::new(0., 13.)),
    }
}
