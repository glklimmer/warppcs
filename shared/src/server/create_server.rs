use bevy::prelude::*;

use bevy_renet::{
    netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::RenetServer,
};

use crate::networking::{connection_config, MultiplayerRoles};

pub fn create_steam_server(mut commands: Commands) {
    use crate::steamworks::SteamworksClient;
    use renet_steam::AccessPermission;
    use renet_steam::SteamServerConfig;
    use renet_steam::SteamServerTransport;

    let server: RenetServer = RenetServer::new(connection_config());
    commands.insert_resource(server);

    commands.queue(|world: &mut World| {
        let steam_client = world.get_resource::<SteamworksClient>().unwrap();
        println!("From Server lib: {}", steam_client.friends().name());
        let steam_transport_config = SteamServerConfig {
            max_clients: 10,
            access_permission: AccessPermission::Public,
        };

        world.insert_non_send_resource(
            SteamServerTransport::new(steam_client, steam_transport_config).unwrap(),
        );
    });
}

pub fn create_netcode_server(
    mut commands: Commands,
    mut multiplayer_roles: ResMut<NextState<MultiplayerRoles>>,
) {
    use crate::networking::PROTOCOL_ID;
    use std::{net::UdpSocket, time::SystemTime};

    let server = RenetServer::new(connection_config());

    let public_addr = "127.0.0.1:5000".parse().unwrap();
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
    commands.insert_resource(server);
    commands.insert_resource(transport);

    multiplayer_roles.set(MultiplayerRoles::Host)
}
