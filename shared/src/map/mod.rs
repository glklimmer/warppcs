use bevy::prelude::*;

use enum_as_f32_macro::enum_as_f32;
use serde::{Deserialize, Serialize};

use crate::BoxCollider;

pub mod buildings;
pub mod scenes;
pub mod spawn_point;

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
pub struct GameSceneId(pub u64);

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum GameSceneType {
    Base(Color),
    Fight,
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
    Chest,
    Unit,
    Projectile,
    Flag,
    Player,
    Wall,
}

#[derive(Component, Clone, Copy)]
pub enum Chest {
    Normal,
    Big,
}

#[derive(Component, Clone, Copy)]
pub enum ChestStatus {
    Closed,
    Open,
}

#[derive(Component, Clone, Copy)]
pub struct ChestTextures {
    pub closed: &'static str,
    pub open: &'static str,
}

#[derive(Bundle, Clone, Copy)]
pub struct ChestBundle {
    pub chest: Chest,
    pub collider: BoxCollider,
    pub chest_status: ChestStatus,
    pub transform: Transform,
}

impl ChestBundle {
    pub fn new(x: f32) -> Self {
        Self {
            chest: Chest::Normal,
            collider: BoxCollider {
                dimension: Vec2::new(50., 35.),
                offset: Some(Vec2::new(0., -30.)),
            },
            chest_status: ChestStatus::Closed,
            transform: Transform::from_xyz(x, 50., Layers::Chest.as_f32())
                .with_scale(Vec3::new(3., 3., 1.)),
        }
    }
}
