use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{Replicated, SendMode, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    AnimationChange, AnimationChangeEvent, BoxCollider,
    networking::{MountType, Mounted},
    server::physics::movement::{Speed, Velocity},
    unit_collider,
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

pub fn mount(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut player_query: Query<&mut Speed>,
    mut commands: Commands,
    mut animation: EventWriter<ToClients<AnimationChangeEvent>>,
    mount_query: Query<&Mount>,
) {
    for event in interactions.read() {
        let InteractionType::Mount = &event.interaction else {
            continue;
        };

        let player = event.player;
        let mut speed = player_query.get_mut(player).unwrap();

        let mount = mount_query.get(event.interactable).unwrap();

        let new_speed = mount_speed(&mount.mount_type);
        speed.0 = new_speed;

        commands.entity(event.interactable).despawn_recursive();
        commands.entity(player).insert(Mounted {
            mount_type: mount.mount_type,
        });

        animation.send(ToClients {
            mode: SendMode::Broadcast,
            event: AnimationChangeEvent {
                entity: player,
                change: AnimationChange::Mount,
            },
        });
    }
}

fn mount_speed(mount_type: &MountType) -> f32 {
    match mount_type {
        MountType::Horse => 150.,
    }
}
