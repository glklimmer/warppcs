use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::Layers;
use crate::BoxCollider;

#[derive(Component, Copy, Clone)]
pub struct RecruitmentBuilding;

#[derive(Component, Copy, Clone)]
pub enum MainBuildingLevel {
    First,
    Second,
    Third,
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildStatus {
    Marker,
    Built,
    Destroyed,
}

#[derive(Component, Copy, Clone)]
pub struct Cost {
    pub gold: u16,
}

#[derive(Component, Copy, Clone, PartialEq, Eq)]
pub enum Building {
    MainBuilding,
    Archer,
    Warrior,
    Pikeman,
    Wall,
    Tower,
    GoldFarm,
}

#[derive(Component, Clone, Copy)]
pub struct BuildingTextures {
    pub marker: &'static str,
    pub built: &'static str,
}

#[derive(Bundle, Copy, Clone)]
pub struct MainBuildingBundle {
    pub base: Building,
    pub collider: BoxCollider,
    pub main_building_level: MainBuildingLevel,
    pub transform: Transform,
}

const BUILDUING_SCALE: Vec3 = Vec3::splat(3.0);

impl MainBuildingBundle {
    pub fn new(x: f32) -> Self {
        MainBuildingBundle {
            base: Building::MainBuilding,
            collider: BoxCollider {
                dimension: Vec2::new(200., 100.),
                offset: None,
            },
            main_building_level: MainBuildingLevel::First,
            transform: Transform::from_xyz(x, 90., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
        }
    }
}

#[derive(Bundle, Clone, Copy)]
pub struct BuildingBundle {
    pub building: Building,
    pub collider: BoxCollider,
    pub build_status: BuildStatus,
    pub transform: Transform,
    pub cost: Cost,
    pub textures: BuildingTextures,
}

impl BuildingBundle {
    pub fn archer(x: f32) -> Self {
        BuildingBundle {
            building: Building::Archer,
            collider: BoxCollider {
                dimension: Vec2::new(200., 100.),
                offset: None,
            },
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
            cost: Cost { gold: 200 },
            textures: BuildingTextures {
                marker: "sprites/buildings/archer_plot.png",
                built: "sprites/buildings/archer_house.png",
            },
        }
    }

    pub fn warrior(x: f32) -> Self {
        BuildingBundle {
            building: Building::Warrior,
            collider: BoxCollider {
                dimension: Vec2::new(200., 100.),
                offset: None,
            },
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
            cost: Cost { gold: 200 },
            textures: BuildingTextures {
                marker: "sprites/buildings/warrior_plot.png",
                built: "sprites/buildings/warrior_house.png",
            },
        }
    }

    pub fn pikeman(x: f32) -> Self {
        BuildingBundle {
            building: Building::Pikeman,
            collider: BoxCollider {
                dimension: Vec2::new(200., 100.),
                offset: None,
            },
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
            cost: Cost { gold: 200 },
            textures: BuildingTextures {
                marker: "sprites/buildings/pike_man_plot.png",
                built: "sprites/buildings/pike_man_house.png",
            },
        }
    }

    pub fn wall(x: f32) -> Self {
        BuildingBundle {
            building: Building::Wall,
            collider: BoxCollider {
                dimension: Vec2::new(50., 75.),
                offset: None,
            },
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 75., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
            cost: Cost { gold: 100 },
            textures: BuildingTextures {
                marker: "sprites/buildings/wall_basic.png",
                built: "sprites/buildings/wall_first_upgrade.png",
            },
        }
    }

    pub fn tower() -> Self {
        BuildingBundle {
            building: Building::Tower,
            collider: BoxCollider {
                dimension: Vec2::new(200., 100.),
                offset: None,
            },
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(0., 50., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
            cost: Cost { gold: 150 },
            textures: BuildingTextures {
                marker: "sprites/buildings/warrior_plot.png",
                built: "sprites/buildings/warrior_house.png",
            },
        }
    }

    pub fn gold_farm(x: f32) -> Self {
        BuildingBundle {
            building: Building::GoldFarm,
            collider: BoxCollider {
                dimension: Vec2::new(200., 50.),
                offset: None,
            },
            build_status: BuildStatus::Marker,
            transform: Transform::from_xyz(x, 25., Layers::Building.as_f32())
                .with_scale(BUILDUING_SCALE),
            cost: Cost { gold: 50 },
            textures: BuildingTextures {
                marker: "sprites/buildings/warrior_plot.png",
                built: "sprites/buildings/warrior_house.png",
            },
        }
    }
}
