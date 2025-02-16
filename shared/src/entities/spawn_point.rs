use bevy::prelude::*;

use crate::{map::scenes::Slot, physics::collider::BoxCollider};

#[derive(Component, Copy, Clone)]
pub struct SpawnPoint;

pub fn spawn_point(x: f32) -> Slot {
    Slot {
        initial_building: None,
        collider: BoxCollider {
            dimension: Vec2::new(100., 100.),
            offset: None,
        },
        transform: Transform::from_xyz(x, 50., 0.),
    }
}
