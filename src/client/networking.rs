use bevy::prelude::*;

use crate::shared::networking::{
    connection_config, ClientChannel, NetworkedEntities, PlayerInput, ServerChannel, ServerMessages,
};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_renet::{
    client_connected,
    renet::{ClientId, RenetClient},
    RenetClientPlugin,
};
use std::collections::HashMap;

pub struct ClientNetworkingPlugin;

impl Plugin for ClientNetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);

        add_netcode_network(app);

        app.insert_resource(NetworkMapping::default());
        app.insert_resource(ClientLobby::default());

        app.add_systems(
            Update,
            (client_sync_players, client_send_input).in_set(Connected),
        );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

#[derive(Debug, Default, Resource)]
struct ClientLobby {
    players: HashMap<ClientId, PlayerInfo>,
}

#[derive(Component)]
struct ControlledPlayer;

#[derive(Debug, Resource)]
struct CurrentClientId(u64);

#[derive(Debug)]
struct PlayerInfo {
    client_entity: Entity,
    server_entity: Entity,
}

#[derive(Default, Resource)]
struct NetworkMapping(HashMap<Entity, Entity>);

fn add_netcode_network(app: &mut App) {
    use crate::shared::networking::PROTOCOL_ID;
    use bevy_renet::renet::transport::{
        ClientAuthentication, NetcodeClientTransport, NetcodeTransportError,
    };
    use std::{net::UdpSocket, time::SystemTime};

    app.add_plugins(bevy_renet::transport::NetcodeClientPlugin);

    app.configure_sets(Update, Connected.run_if(client_connected));

    let client = RenetClient::new(connection_config());

    let server_addr = "127.0.0.1:6969".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    app.insert_resource(client);
    app.insert_resource(transport);
    app.insert_resource(CurrentClientId(client_id));

    // If any error is found we just panic
    #[allow(clippy::never_loop)]
    fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
        for e in renet_error.read() {
            panic!("{}", e);
        }
    }

    app.add_systems(Update, panic_on_error_system);
}

fn client_sync_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut client: ResMut<RenetClient>,
    client_id: Res<CurrentClientId>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let client_id = client_id.0;
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerCreate {
                id,
                translation,
                entity,
                color,
            } => {
                println!("Player {} connected.", id);
                let mut client_entity = commands.spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Capsule2d::new(25.0, 50.0))),
                    material: materials.add(color),
                    transform: Transform::from_xyz(translation[0], translation[1], translation[2]),
                    ..default()
                });

                if client_id == id.raw() {
                    client_entity.insert(ControlledPlayer);
                }

                let player_info = PlayerInfo {
                    server_entity: entity,
                    client_entity: client_entity.id(),
                };
                lobby.players.insert(id, player_info);
                network_mapping.0.insert(entity, client_entity.id());
            }
            ServerMessages::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                if let Some(PlayerInfo {
                    server_entity,
                    client_entity,
                }) = lobby.players.remove(&id)
                {
                    commands.entity(client_entity).despawn();
                    network_mapping.0.remove(&server_entity);
                }
            }
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();

        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping.0.get(&networked_entities.entities[i]) {
                let translation = networked_entities.translations[i].into();
                let transform = Transform {
                    translation,
                    ..Default::default()
                };
                commands.entity(*entity).insert(transform);
            }
        }
    }
}

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();
    client.send_message(ClientChannel::Input, input_message);
}
