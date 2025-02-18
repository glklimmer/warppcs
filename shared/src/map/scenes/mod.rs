use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{entities::Owner, networking::SlotType, physics::collider::BoxCollider};

pub mod base;
pub mod camp;

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
pub struct GameSceneId(pub usize);

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum GameSceneType {
    Base,
    Camp,
}

pub struct GameScene {
    pub game_scene_type: GameSceneType,
    pub slots: Vec<SlotPrefab>,
    pub left_portal: SlotPrefab,
    pub right_portal: SlotPrefab,
}

pub struct SlotPrefab {
    pub slot_type: SlotType,
    pub transform: Transform,
    pub collider: BoxCollider,
    pub spawn_fn: fn(&mut EntityCommands, Owner),
}
