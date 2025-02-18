use bevy::prelude::*;

use crate::{
    map::{scenes::SlotPrefab, Layers},
    physics::collider::BoxCollider,
    server::players::{
        interaction::{Interactable, InteractionType},
        mount::Mount,
    },
};

use super::MountType;

pub fn horse(x: f32) -> SlotPrefab {
    SlotPrefab {
        collider: BoxCollider {
            dimension: Vec2::new(40., 35.),
            offset: Some(Vec2::new(0., -28.)),
        },
        transform: Transform::from_xyz(x, 45., Layers::Unit.as_f32()),
        spawn_fn: |entity, _| {
            entity.insert((
                Mount {
                    mount_type: MountType::Horse,
                },
                Interactable {
                    kind: InteractionType::Mount,
                    restricted_to: None,
                },
            ));
        },
    }
}
