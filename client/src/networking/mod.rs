use bevy::prelude::*;

use bevy_renet::{
    client_connected,
    renet::{ClientId, RenetClient},
    RenetClientPlugin,
};
use shared::networking::{
    ClientChannel, NetworkedEntities, PlayerCommand, PlayerInput, ServerChannel, ServerMessages,
};
use std::collections::HashMap;

use crate::animations::animation::{Change, EntityChangeEvent};

pub mod join_server;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Connected;

#[derive(Debug, Default, Resource)]
pub struct ClientPlayers {
    pub players: HashMap<ClientId, PlayerEntityMapping>,
}

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Debug, Resource)]
pub struct CurrentClientId(pub ClientId);

#[derive(Debug)]
pub struct PlayerEntityMapping {
    pub client_entity: Entity,
    pub server_entity: Entity,
}

#[derive(Default, Resource)]
pub struct NetworkMapping(pub HashMap<Entity, Entity>);

#[derive(Event)]
pub struct NetworkEvent {
    pub message: ServerMessages,
}

pub struct ClientNetworkPlugin;

impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);

        app.insert_resource(NetworkMapping::default());
        app.insert_resource(ClientPlayers::default());

        app.add_event::<NetworkEvent>();

        app.add_systems(
            FixedPreUpdate,
            (recieve_server_messages, recieve_networked_entities)
                .run_if(client_connected)
                .in_set(Connected),
        );

        app.add_systems(Update, (send_input, send_player_commands).in_set(Connected));
    }
}

fn recieve_server_messages(
    mut client: ResMut<RenetClient>,
    mut network_event: EventWriter<NetworkEvent>,
) {
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let message: ServerMessages = bincode::deserialize(&message).unwrap();
        network_event.send(NetworkEvent { message });
    }
}

fn recieve_networked_entities(
    mut client: ResMut<RenetClient>,
    mut change_events: EventWriter<EntityChangeEvent>,
    mut transforms: Query<&mut Transform>,
    network_mapping: Res<NetworkMapping>,
) {
    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let maybe_net_entities: Result<NetworkedEntities, _> = bincode::deserialize(&message);
        match maybe_net_entities {
            Ok(networked_entities) => {
                for i in 0..networked_entities.entities.len() {
                    if let Some(client_entity) = network_mapping
                        .0
                        .get(&networked_entities.entities[i].entity)
                    {
                        let network_entity = &networked_entities.entities[i];

                        if let Ok(mut transform) = transforms.get_mut(*client_entity) {
                            transform.translation = network_entity.translation.into();
                        }

                        change_events.send(EntityChangeEvent {
                            entity: *client_entity,
                            change: Change::Rotation(network_entity.rotation.clone()),
                        });

                        change_events.send(EntityChangeEvent {
                            entity: *client_entity,
                            change: Change::Movement(network_entity.moving),
                        });
                    }
                }
            }
            Err(error) => error!("Error on deserialize: {}", error),
        }
    }
}

fn send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();
    client.send_message(ClientChannel::Input, input_message);
}

fn send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.read() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command, command_message);
    }
}
