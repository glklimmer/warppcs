use bevy::prelude::*;

use bevy_renet::renet::ClientId;

use crate::{
    map::GameSceneId,
    networking::{PlayerCommand, ServerMessages},
};

use super::networking::{NetworkEvent, SendServerMessage, ServerLobby};

#[derive(Event)]
pub struct InteractEvent(pub ClientId);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractEvent>();

        app.add_systems(
            FixedUpdate,
            (attack, interact).run_if(on_event::<NetworkEvent>()),
        );
    }
}

fn attack(
    mut network_events: EventReader<NetworkEvent>,
    mut sender: EventWriter<SendServerMessage>,
    scene_ids: Query<&GameSceneId>,
    lobby: Res<ServerLobby>,
) {
    for event in network_events.read() {
        let client_id = event.client_id;
        if let PlayerCommand::MeleeAttack = &event.message {
            if let Some(player_entity) = lobby.players.get(&client_id) {
                let game_scene_id = scene_ids.get(*player_entity).unwrap();
                sender.send(SendServerMessage {
                    message: ServerMessages::MeleeAttack {
                        entity: *player_entity,
                    },
                    game_scene_id: *game_scene_id,
                });
            }
        }
    }
}

fn interact(
    mut network_events: EventReader<NetworkEvent>,
    mut interact: EventWriter<InteractEvent>,
) {
    for event in network_events.read() {
        let client_id = event.client_id;
        if let PlayerCommand::Interact = &event.message {
            interact.send(InteractEvent(client_id));
        }
    }
}
