use bevy::prelude::*;

use crate::{
    map::GameSceneId,
    networking::{MountType, ServerMessages},
    server::{networking::SendServerMessage, physics::movement::Speed},
};

use super::interaction::{InteractionTriggeredEvent, InteractionType};

pub fn mount(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut player_query: Query<(&mut Speed, &GameSceneId)>,
    mut commands: Commands,
    mut sender: EventWriter<SendServerMessage>,
    mount_query: Query<&MountType>,
) {
    for event in interactions.read() {
        let InteractionType::Mount = &event.interaction else {
            continue;
        };

        let (mut speed, scene_id) = player_query.get_mut(event.player).unwrap();
        let mount_type = mount_query.get(event.interactable).unwrap();

        let new_speed = mount_speed(mount_type);
        speed.0 = new_speed;

        commands.entity(event.interactable).despawn();

        sender.send(SendServerMessage {
            message: ServerMessages::Mount {
                entity: event.player,
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
