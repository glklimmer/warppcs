use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{entities::Owner, physics::collider::BoxCollider};

pub mod base;
pub mod camp;
pub mod fight;

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
pub struct GameSceneId(pub usize);

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
    pub left_portal: SceneSlot,
    pub right_portal: SceneSlot,
}

pub struct SceneSlot {
    pub transform: Transform,
    pub collider: BoxCollider,
    pub spawn_fn: fn(&mut EntityCommands, Owner),
}
