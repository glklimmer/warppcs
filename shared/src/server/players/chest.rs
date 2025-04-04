use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{Replicated, SendMode, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    networking::MountType, server::physics::movement::Velocity, unit_collider, BoxCollider,
    ChestAnimation, ChestAnimationEvent,
};

use super::interaction::{Interactable, InteractionTriggeredEvent, InteractionType};

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider(unit_collider),
    Velocity,
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
    Interactable(|| Interactable {
        kind: InteractionType::Mount,
        restricted_to: None,
    }),
)]
pub struct Mount {
    pub mount_type: MountType,
}

#[derive(Component, Clone, Copy)]
#[require(
    Replicated,
    Transform,
    BoxCollider(chest_collider),
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
    Interactable(|| Interactable {
        kind: InteractionType::Chest,
        restricted_to: None,
    }),
)]
pub enum Chest {
    Normal,
    Big,
}

#[derive(Component, Clone, Copy)]
pub enum ChestStatus {
    Closed,
    Open,
}

pub fn open_chest(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    mut animation: EventWriter<ToClients<ChestAnimationEvent>>,
) {
    for event in interactions.read() {
        let InteractionType::Chest = &event.interaction else {
            continue;
        };

        commands.entity(event.interactable).remove::<Interactable>();

        animation.send(ToClients {
            mode: SendMode::Broadcast,
            event: ChestAnimationEvent {
                entity: event.interactable,
                animation: ChestAnimation::Open,
            },
        });
    }
}
fn chest_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 10.),
        offset: Some(Vec2::new(0., -5.)),
    }
}
