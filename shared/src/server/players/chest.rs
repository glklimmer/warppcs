use bevy::prelude::*;

use crate::{
    map::GameSceneId,
    networking::ServerMessages,
    server::{
        networking::SendServerMessage,
        players::interaction::{InteractionTriggeredEvent, InteractionType},
    },
};

pub fn open_chest(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    mut sender: EventWriter<SendServerMessage>,
) {
    for event in interactions.read() {
        let InteractionType::Chest = &event.interaction else {
            continue;
        };

        println!("chest open");

        // sender.send(SendServerMessage {
        //     message: ServerMessages::Mount {
        //         entity: event.player,
        //         mount_type: mount.mount_type,
        //     },
        //     game_scene_id: *scene_id,
        // });
    }
}
