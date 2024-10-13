use bevy::prelude::*;

use bevy_renet::renet::RenetServer;

use crate::networking::connection_config;

pub fn create_steam_server(mut commands: Commands) {
    use crate::steamworks::SteamworksClient;
    use renet_steam::bevy::{SteamServerConfig, SteamServerTransport};
    use renet_steam::AccessPermission;

    let server: RenetServer = RenetServer::new(connection_config());
    commands.insert_resource(server);

    commands.add(|world: &mut World| {
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

pub fn create_netcode_server(mut commands: Commands) {
    use crate::networking::PROTOCOL_ID;
    use bevy_renet::renet::transport::{
        NetcodeServerTransport, ServerAuthentication, ServerConfig,
    };
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
}
