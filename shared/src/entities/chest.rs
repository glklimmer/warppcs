use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{
    map::scenes::SlotPrefab,
    networking::SlotType,
    physics::collider::BoxCollider,
    server::players::interaction::{Interactable, InteractionType},
};

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

pub trait Prefab {
    fn prefab() -> SlotPrefab;
}

impl SlotType for Prefab {
    fn prefab(&self, x: f32) -> SlotPrefab {
        SlotPrefab {
            slot_type: SlotType::Chest,
            collider: BoxCollider {
                dimension: Vec2::new(100., 100.),
                offset: None,
            },
            transform: Transform::from_xyz(x, 50., 0.),
            spawn_fn: |entity, _| {
                entity.insert((
                    Chest::Normal,
                    ChestStatus::Closed,
                    Interactable {
                        kind: InteractionType::Chest,
                        restricted_to: None,
                    },
                ));
            },
        }
    }
}
