use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{AppRuleExt, Replicated};
use gold_farm::{enable_goldfarm, gold_farm_output};
use health::Health;
use inventory::Cost;
use item_assignment::ItemAssignmentPlugins;
use lobby::PlayerColor;
use physics::movement::BoxCollider;
use respawn::respawn_units;
use serde::{Deserialize, Serialize};
use shared::{GameState, enum_map::*};
use units::UnitType;

use crate::{
    animations::BuildingAnimationPlugin,
    construction::{BuildingChangeEnd, ConstructionPlugins},
    destruction::DestructionPlugin,
    main_building::MainBuildingLevels,
    recruiting::RecruitingPlugins,
    respawn::{RespawnZone, respawn_timer},
    siege_camp::SiegeCampPlugin,
    wall::{WallLevels, WallPlugin},
};

mod animations;
mod construction;
mod destruction;

pub mod gold_farm;
pub mod item_assignment;
pub mod main_building;
pub mod recruiting;
pub mod respawn;
pub mod siege_camp;
pub mod wall;

pub struct BuildingsPlugins;

impl Plugin for BuildingsPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ItemAssignmentPlugins,
            ConstructionPlugins,
            DestructionPlugin,
            BuildingAnimationPlugin,
            RecruitingPlugins,
            WallPlugin,
            SiegeCampPlugin,
        ))
        .replicate_bundle::<(Building, BuildStatus, Transform)>()
        .replicate_bundle::<(RespawnZone, Transform)>()
        .add_systems(
            FixedUpdate,
            (
                gold_farm_output.run_if(in_state(GameState::GameSession)),
                (respawn_timer, respawn_units).chain(),
                enable_goldfarm.run_if(on_message::<BuildingChangeEnd>),
            ),
        );
    }
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

#[derive(Component, Debug, Copy, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    BoxCollider = marker_collider(),
    Sprite,
    Anchor::BOTTOM_CENTER,
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
