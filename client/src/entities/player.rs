use bevy::prelude::*;

use crate::networking::{
    ClientPlayers, Connected, NetworkEvent, NetworkMapping, PlayerEntityMapping,
};
use shared::networking::ServerMessages;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (remove_player)
                .run_if(on_event::<NetworkEvent>())
                .in_set(Connected),
        );
    }
}

fn remove_player(
    mut commands: Commands,
    mut network_mapping: ResMut<NetworkMapping>,
    mut network_events: EventReader<NetworkEvent>,
    mut lobby: ResMut<ClientPlayers>,
) {
    for event in network_events.read() {
        if let ServerMessages::PlayerDisconnected { id } = &event.message {
            println!("Player {} disconnected.", id);
            if let Some(PlayerEntityMapping {
                server_entity,
                client_entity,
            }) = lobby.players.remove(id)
            {
                commands.entity(client_entity).despawn();
                network_mapping.0.remove(&server_entity);
            }
        }
    }
}
