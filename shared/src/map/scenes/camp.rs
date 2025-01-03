use bevy::prelude::*;

use crate::map::{spawn_point::SpawnPointBundle, ChestBundle};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct CampScene {
    pub chest: ChestBundle,
    pub left_spawn_point: SpawnPointBundle,
    pub right_spawn_point: SpawnPointBundle,
}

#[derive(Copy, Clone, Component, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum CampSceneIndicator {
    Chest,
    LeftSpawn,
    RightSpawn,
}

impl CampScene {
    pub fn new() -> Self {
        Self {
            chest: ChestBundle::new(0.),
            left_spawn_point: SpawnPointBundle::new(-600.),
            right_spawn_point: SpawnPointBundle::new(600.),
        }
    }
}

impl Default for CampScene {
    fn default() -> Self {
        Self::new()
    }
}
