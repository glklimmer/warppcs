use bevy::prelude::*;

use enum_as_f32_macro::enum_as_f32;
use serde::{Deserialize, Serialize};

pub mod base;
pub mod spawn_point;

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
pub struct GameSceneId(pub u64);

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum GameSceneType {
    Base(Color),
    Camp,
}

pub struct GameScene {
    pub id: GameSceneId,
    pub game_scene_type: GameSceneType,
    pub left_game_scenes: Vec<GameSceneId>,
    pub right_game_scenes: Vec<GameSceneId>,
}

#[enum_as_f32]
#[derive(Component)]
pub enum Layers {
    Background,
    Building,
    Unit,
    Projectile,
    Flag,
    Player,
}

#[derive(Component)]
pub struct Goldmine;
