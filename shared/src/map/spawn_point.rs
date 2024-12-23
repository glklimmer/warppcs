use bevy::{math::bounding::Aabb2d, prelude::*};

use crate::BoxCollider;

#[derive(Component, Copy, Clone)]
pub struct SpawnPoint;

#[derive(Bundle, Copy, Clone)]
pub struct SpawnPointBundle {
    pub spawn_point: SpawnPoint,
    pub collider: BoxCollider,
    pub transform: Transform,
}

#[derive(Component, Clone)]
pub struct SpawnPointBound {
    pub bound: Aabb2d,
}

impl SpawnPointBundle {
    pub fn new(x: f32) -> Self {
        SpawnPointBundle {
            spawn_point: SpawnPoint,
            collider: BoxCollider(Vec2::new(100., 100.)),
            transform: Transform::from_xyz(x, 50., 0.),
        }
    }
}
