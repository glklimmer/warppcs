use bevy::prelude::*;

use crate::{
    map::GameSceneId,
    networking::{MountType, Mounted, ServerMessages},
    server::{
        networking::SendServerMessage,
        physics::movement::{Speed, Velocity},
    },
    unit_collider, BoxCollider,
};

use super::interaction::{InteractionTriggeredEvent, InteractionType};

#[derive(Component, Clone)]
#[require(BoxCollider(unit_collider), Velocity)]
pub struct Mount {
    pub mount_type: MountType,
}

pub fn mount(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut player_query: Query<(&mut Speed, &GameSceneId)>,
    mut commands: Commands,
    mut sender: EventWriter<SendServerMessage>,
    mount_query: Query<&Mount>,
) {
    for event in interactions.read() {
        let InteractionType::Mount = &event.interaction else {
            continue;
        };

        let (mut speed, scene_id) = player_query.get_mut(event.player).unwrap();
        let mount = mount_query.get(event.interactable).unwrap();

        let new_speed = mount_speed(&mount.mount_type);
        speed.0 = new_speed;

        commands.entity(event.interactable).despawn();
        commands.entity(event.player).insert(Mounted {
            mount_type: mount.mount_type,
        });

        sender.send(SendServerMessage {
            message: ServerMessages::DespawnEntity {
                entities: vec![event.interactable],
            },
            game_scene_id: *scene_id,
        });

        sender.send(SendServerMessage {
            message: ServerMessages::Mount {
                entity: event.player,
                mount_type: mount.mount_type,
            },
            game_scene_id: *scene_id,
        });
    }
}

fn mount_speed(mount_type: &MountType) -> f32 {
    match mount_type {
        MountType::Horse => 450.,
    }
}
