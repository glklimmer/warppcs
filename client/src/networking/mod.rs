use bevy::prelude::*;

use bevy_renet::renet::ClientId;
use shared::{networking::ServerMessages, player_collider, BoxCollider};
use std::collections::HashMap;

use crate::animations::king::KingAnimation;

pub mod join_server;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Connected;

#[derive(Debug, Default, Resource)]
pub struct ClientPlayers {
    pub players: HashMap<ClientId, PlayerEntityMapping>,
}

#[derive(Component)]
#[require(BoxCollider(player_collider), KingAnimation)]
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
        // app.insert_resource(NetworkMapping::default());
        // app.insert_resource(ClientPlayers::default());
        //
        // app.add_event::<NetworkEvent>();
        //
        // app.add_systems(
        //     FixedPreUpdate,
        //     (recieve_server_messages, recieve_networked_entities)
        //         .run_if(client_connected)
        //         .in_set(Connected),
        // );
        //
        // app.add_systems(
        //     Update,
        //     (
        //         send_input.run_if(resource_changed::<PlayerInput>),
        //         send_player_commands.run_if(on_event::<PlayerCommand>),
        //     )
        //         .in_set(Connected),
        // );
    }
}

// fn recieve_networked_entities(
//     mut client: ResMut<RenetClient>,
//     mut change_events: EventWriter<EntityChangeEvent>,
//     mut transforms: Query<&mut Transform>,
//     network_mapping: Res<NetworkMapping>,
// ) {
//     while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
//         let maybe_net_entities: Result<NetworkedEntities, _> = bincode::deserialize(&message);
//         match maybe_net_entities {
//             Ok(networked_entities) => {
//                 for i in 0..networked_entities.entities.len() {
//                     if let Some(client_entity) = network_mapping
//                         .0
//                         .get(&networked_entities.entities[i].entity)
//                     {
//                         let network_entity = &networked_entities.entities[i];
//
//                         if let Ok(mut transform) = transforms.get_mut(*client_entity) {
//                             transform.translation = network_entity.translation.into();
//                         }
//
//                         change_events.send(EntityChangeEvent {
//                             entity: *client_entity,
//                             change: Change::Rotation(network_entity.rotation.clone()),
//                         });
//
//                         change_events.send(EntityChangeEvent {
//                             entity: *client_entity,
//                             change: Change::Movement(network_entity.moving),
//                         });
//                     }
//                 }
//             }
//             Err(error) => error!("Error on deserialize: {}", error),
//         }
//     }
// }
