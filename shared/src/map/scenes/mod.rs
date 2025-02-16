use bevy::prelude::*;

use base::BaseSceneIndicator;
use camp::CampSceneIndicator;
use fight::FightSceneIndicator;
use serde::{Deserialize, Serialize};

use crate::physics::collider::BoxCollider;

use super::buildings::{BuildStatus, Building};

pub mod base;
pub mod camp;
pub mod fight;

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
pub struct GameSceneId(pub u64);

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum GameSceneType {
    Base,
    Fight,
    Camp,
}

pub struct GameScene {
    pub id: GameSceneId,
    pub game_scene_type: GameSceneType,
    pub slots: Vec<SceneSlot>,
    pub left_game_scenes: Vec<GameSceneId>,
    pub right_game_scenes: Vec<GameSceneId>,
}

#[derive(Copy, Clone, Component, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SceneSlotIndicator {
    Base(BaseSceneIndicator),
    Fight(FightSceneIndicator),
    Camp(CampSceneIndicator),
}

pub struct SceneSlot {
    pub indicator: SceneSlotIndicator,
    pub slot: Slot,
}

pub struct Slot {
    pub initial_building: Option<(Building, BuildStatus)>,
    pub transform: Transform,
    pub collider: BoxCollider,
}
