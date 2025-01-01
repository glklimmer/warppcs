use bevy::prelude::*;

use crate::BoxCollider;

#[derive(Component, Copy, Clone)]
pub struct SpawnPoint;

#[derive(Bundle, Copy, Clone)]
pub struct SpawnPointBundle {
    pub spawn_point: SpawnPoint,
    pub collider: BoxCollider,
    pub transform: Transform,
}

impl SpawnPointBundle {
    pub fn new(x: f32) -> Self {
        SpawnPointBundle {
            spawn_point: SpawnPoint,
            collider: BoxCollider {
                dimension: Vec2::new(100., 100.),
                offset: None,
            },
            transform: Transform::from_xyz(x, 50., 0.),
        }
    }
}
