use bevy::prelude::*;

use crate::{
    map::scenes::SceneSlot,
    physics::collider::BoxCollider,
    server::players::interaction::{Interactable, InteractionType},
};

#[derive(Component, Copy, Clone)]
pub struct SpawnPoint;

pub fn spawn_point(x: f32) -> SceneSlot {
    SceneSlot {
        collider: BoxCollider {
            dimension: Vec2::new(100., 100.),
            offset: None,
        },
        transform: Transform::from_xyz(x, 50., 0.),
        spawn_fn: |entity, _| {
            entity.insert((
                SpawnPoint,
                Interactable {
                    kind: InteractionType::Travel,
                    restricted_to: None,
                },
            ));
        },
    }
}
