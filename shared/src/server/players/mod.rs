use bevy::prelude::*;

use bevy_renet::renet::{ClientId, RenetServer};

use crate::networking::{PlayerCommand, ServerChannel, ServerMessages};

use super::networking::{NetworkEvent, ServerLobby};

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
    mut server: ResMut<RenetServer>,
    lobby: Res<ServerLobby>,
) {
    for event in network_events.read() {
        let client_id = event.client_id;
        if let PlayerCommand::MeleeAttack = &event.message {
            if let Some(player_entity) = lobby.players.get(&client_id) {
                let message = ServerMessages::MeleeAttack {
                    entity: *player_entity,
                };
                let message = bincode::serialize(&message).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
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
