use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{Replicated, SendMode, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, ChestAnimation, ChestAnimationEvent, Vec3LayerExt, map::Layers,
    networking::MountType, server::physics::movement::Velocity, unit_collider,
};

use super::{
    interaction::{Interactable, InteractionTriggeredEvent, InteractionType},
    items::{Item, Rarity},
};

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

#[derive(Component, Clone, Copy, Serialize, Deserialize)]
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
    query: Query<&Transform>,
) {
    for event in interactions.read() {
        let InteractionType::Chest = &event.interaction else {
            continue;
        };

        commands.entity(event.interactable).remove::<Interactable>();
        let chest_transform = query.get(event.interactable).unwrap();
        let chest_translation = chest_transform.translation;

        let item = Item::random(Rarity::Common);
        info!("Spawning item: {:?}", item);
        commands.spawn((
            item.collider(),
            item,
            chest_translation.with_y(12.5).with_layer(Layers::Item),
            Velocity(Vec2::new((fastrand::f32() - 0.5) * 50., 50.)),
        ));

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
        offset: Some(Vec2::new(0., 5.)),
    }
}
