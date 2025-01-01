use bevy::prelude::*;

use crate::networking::{Connected, NetworkEvent, NetworkMapping};
use shared::networking::ServerMessages;

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (despawn_entity)
                .run_if(on_event::<NetworkEvent>)
                .in_set(Connected),
        );
    }
}

fn despawn_entity(
    mut commands: Commands,
    mut network_mapping: ResMut<NetworkMapping>,
    mut network_events: EventReader<NetworkEvent>,
) {
    for event in network_events.read() {
        if let ServerMessages::DespawnEntity {
            entities: server_entities,
        } = &event.message
        {
            for server_entity in server_entities {
                if let Some(client_entity) = network_mapping.0.remove(server_entity) {
                    if let Some(mut entity) = commands.get_entity(client_entity) {
                        entity.despawn();
                    }
                }
            }
        }
    }
}
