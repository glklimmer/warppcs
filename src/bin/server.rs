use std::collections::HashMap;

use bevy::prelude::*;
use bevy_renet::{
    renet::{ClientId, RenetServer, ServerEvent},
    RenetServerPlugin,
};
use warppcs::{ClientChannel, NetworkedEntities, PlayerInput, ServerChannel, ServerMessages};

#[derive(Debug, Component)]
pub struct Player {
    pub id: ClientId,
    pub color: Color,
}

#[derive(Debug, Default, Component)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<ClientId, Entity>,
}

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    app.add_plugins(RenetServerPlugin);
    app.insert_resource(ServerLobby::default());

    add_netcode_network(&mut app);

    app.add_systems(Update, (server_update_system, server_network_sync));

    app.run();
}

fn add_netcode_network(app: &mut App) {
    use bevy_renet::renet::transport::{
        NetcodeServerTransport, ServerAuthentication, ServerConfig,
    };
    use bevy_renet::transport::NetcodeServerPlugin;
    use std::{net::UdpSocket, time::SystemTime};
    use warppcs::{connection_config, PROTOCOL_ID};

    app.add_plugins(NetcodeServerPlugin);

    let server = RenetServer::new(connection_config());

    let public_addr = "127.0.0.1:6969".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time: std::time::Duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    app.insert_resource(server);
    app.insert_resource(transport);
}

fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    players: Query<(Entity, &Player, &Transform)>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {} connected.", client_id);
                println!("test");

                players.iter().for_each(|player| {
                    println!(
                        "Sending player {} with color {}.",
                        player.1.id,
                        player.1.color.hue()
                    );
                });

                // Initialize other players for this new client
                for (entity, player, transform) in players.iter() {
                    let translation: [f32; 3] = transform.translation.into();
                    let message = bincode::serialize(&ServerMessages::PlayerCreate {
                        id: player.id,
                        entity,
                        translation,
                        color: player.color,
                    })
                    .unwrap();
                    server.send_message(*client_id, ServerChannel::ServerMessages, message);
                }

                // Spawn new player
                let transform = Transform::from_xyz(
                    (fastrand::f32() - 0.5) * 200.,
                    0.51,
                    (fastrand::f32() - 0.5) * 200.,
                );
                let color = Color::hsl(fastrand::f32() * 360., 0.8, 0.8);
                let player_entity = commands
                    .spawn((
                        transform,
                        PlayerInput::default(),
                        Velocity::default(),
                        Player {
                            id: *client_id,
                            color,
                        },
                    ))
                    .id();

                lobby.players.insert(*client_id, player_entity);

                let translation: [f32; 3] = transform.translation.into();
                let message = bincode::serialize(&ServerMessages::PlayerCreate {
                    id: *client_id,
                    entity: player_entity,
                    translation,
                    color,
                })
                .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Player {} disconnected: {}", client_id, reason);

                if let Some(player_entity) = lobby.players.remove(client_id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerRemove { id: *client_id }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let input: PlayerInput = bincode::deserialize(&message).unwrap();
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(input);
            }
        }
    }
}

fn server_network_sync(mut server: ResMut<RenetServer>, query: Query<(Entity, &Transform)>) {
    let mut networked_entities = NetworkedEntities::default();
    for (entity, transform) in query.iter() {
        networked_entities.entities.push(entity);
        networked_entities
            .translations
            .push(transform.translation.into());
    }

    let sync_message = bincode::serialize(&networked_entities).unwrap();
    server.broadcast_message(ServerChannel::NetworkedEntities, sync_message);
}
