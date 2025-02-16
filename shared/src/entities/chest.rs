use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{map::scenes::Slot, physics::collider::BoxCollider};

#[derive(Component, Clone, Copy, Reflect, Serialize, Deserialize)]
#[require(ChestStatus)]
pub enum Chest {
    Normal,
    Big,
}

#[derive(Default, Component, Clone, Copy, Reflect, Serialize, Deserialize)]
pub enum ChestStatus {
    #[default]
    Closed,
    Open,
}

pub fn chest(x: f32) -> Slot {
    Slot {
        initial_building: None,
        collider: BoxCollider {
            dimension: Vec2::new(100., 100.),
            offset: None,
        },
        transform: Transform::from_xyz(x, 50., 0.),
    }
}
