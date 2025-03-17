use bevy::prelude::*;
use bevy_replicon::prelude::*;

use bevy_renet::{
    netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::{ConnectionConfig, RenetServer},
};
use bevy_replicon::prelude::RepliconChannels;
use bevy_replicon_renet::RenetChannelsExt;

use crate::{server::networking::Player, PhysicalPlayer};

pub fn create_steam_server(mut commands: Commands, channels: Res<RepliconChannels>) {
    use crate::steamworks::SteamworksClient;
    use renet_steam::AccessPermission;
    use renet_steam::SteamServerConfig;
    use renet_steam::SteamServerTransport;

    let server_channels_config = channels.get_server_configs();
    let client_channels_config = channels.get_client_configs();

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

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

pub fn create_netcode_server(mut commands: Commands, channels: Res<RepliconChannels>) {
    use crate::networking::PROTOCOL_ID;
    use std::{net::UdpSocket, time::SystemTime};

    let server_channels_config = channels.get_server_configs();
    let client_channels_config = channels.get_client_configs();

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

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

    commands.spawn(Player(ClientId::SERVER));
    commands.spawn(PhysicalPlayer(ClientId::SERVER));
    println!("Successfully started server.")
}
