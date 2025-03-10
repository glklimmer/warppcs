use bevy::prelude::*;

use bevy_renet::renet::{ConnectionConfig, RenetClient};
use bevy_replicon::prelude::RepliconChannels;
use bevy_replicon_renet::RenetChannelsExt;

use crate::networking::CurrentClientId;

#[cfg(feature = "steam")]
use crate::menu::{JoinSteamLobby, MainMenuStates};

#[cfg(feature = "steam")]
use shared::steamworks::SteamworksClient;

#[cfg(feature = "steam")]
pub fn join_steam_server(
    mut commands: Commands,
    mut join_lobby: EventReader<JoinSteamLobby>,
    mut ui: ResMut<NextState<MainMenuStates>>,
    steam_client: Res<SteamworksClient>,
    channels: Res<RepliconChannels>,
) {
    let server_steam_id = match join_lobby.read().next() {
        Some(value) => value.0,
        None => return,
    };

    use renet_steam::SteamClientTransport;

    let server_channels_config = channels.get_server_configs();
    let client_channels_config = channels.get_client_configs();

    let client = RenetClient::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    steam_client.networking_utils().init_relay_network_access();

    println!("From Client {}", steam_client.friends().name());

    match SteamClientTransport::new(&steam_client, &server_steam_id) {
        Ok(transport) => {
            commands.insert_resource(transport);
            commands.insert_resource(client);
            commands.insert_resource(CurrentClientId(steam_client.user().steam_id().raw()));
            ui.set(MainMenuStates::Lobby);
        }
        Err(error) => println!("join_netcode_server error {}", error),
    }
}

#[cfg(feature = "netcode")]
pub fn join_netcode_server(mut commands: Commands, channels: Res<RepliconChannels>) {
    use bevy_renet::netcode::{ClientAuthentication, NetcodeClientTransport};
    use shared::networking::PROTOCOL_ID;
    use std::{net::UdpSocket, time::SystemTime};

    let server_channels_config = channels.get_server_configs();
    let client_channels_config = channels.get_client_configs();

    let client = RenetClient::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr: "127.0.0.1:5000".parse().unwrap(),
        user_data: None,
    };

    match NetcodeClientTransport::new(current_time, authentication, socket) {
        Ok(transport) => {
            commands.insert_resource(client);
            commands.insert_resource(transport);
            commands.insert_resource(CurrentClientId(client_id));
        }
        Err(error) => println!("join_netcode_server error {}", error),
    }
}
