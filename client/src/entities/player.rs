use bevy::prelude::*;

use crate::{
    animations::{king::KingAnimation, FullAnimation},
    networking::{ClientPlayers, Connected, NetworkEvent, NetworkMapping, PlayerEntityMapping},
};
use shared::networking::ServerMessages;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (remove_player, mount_player)
                .run_if(on_event::<NetworkEvent>)
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

fn mount_player(
    mut commands: Commands,
    mut network_events: EventReader<NetworkEvent>,
    network_mapping: ResMut<NetworkMapping>,
) {
    for event in network_events.read() {
        let ServerMessages::Mount {
            entity: server_entity,
        } = &event.message
        else {
            continue;
        };

        let player = network_mapping.0.get(server_entity).unwrap();
        commands
            .entity(*player)
            .insert((KingAnimation::Mount, FullAnimation));
    }
}
