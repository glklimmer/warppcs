use bevy::prelude::*;

use bevy_renet::renet::{ClientId, RenetClient};
use shared::networking::connection_config;

#[cfg(feature = "steam")]
use crate::menu::JoinSteamLobby;
#[cfg(prod)]
use crate::menu::MainMenuStates;
use crate::networking::CurrentClientId;

#[cfg(feature = "netcode")]
use crate::menu::JoinNetcodeLobby;

#[cfg(feature = "steam")]
use shared::steamworks::SteamworksClient;

#[cfg(feature = "steam")]
pub fn join_own_steam_server(
    mut join_lobby: EventWriter<JoinSteamLobby>,
    steam_client: Res<SteamworksClient>,
) {
    join_lobby.send(JoinSteamLobby(steam_client.user().steam_id()));
}

#[cfg(feature = "steam")]
pub fn join_steam_server(
    mut commands: Commands,
    steam_client: Res<SteamworksClient>,
    mut join_lobby: EventReader<JoinSteamLobby>,
    #[cfg(prod)] mut ui: ResMut<NextState<MainMenuStates>>,
) {
    let server_steam_id = match join_lobby.read().next() {
        Some(value) => value.0,
        None => return,
    };

    use renet_steam::SteamClientTransport;

    let client = RenetClient::new(connection_config());

    steam_client.networking_utils().init_relay_network_access();

    println!("From Client {}", steam_client.friends().name());

    match SteamClientTransport::new(&steam_client, &server_steam_id) {
        Ok(transport) => {
            commands.insert_resource(transport);
            commands.insert_resource(client);
            commands.insert_resource(CurrentClientId(ClientId::from_raw(
                steam_client.user().steam_id().raw(),
            )));
            #[cfg(prod)]
            ui.set(MainMenuStates::Lobby);
        }
        Err(error) => println!("join_netcode_server error {}", error),
    }
}

#[cfg(feature = "netcode")]
pub fn join_own_netcode_server(mut join_lobby: EventWriter<JoinNetcodeLobby>) {
    join_lobby.send(JoinNetcodeLobby("127.0.0.1:5000".parse().unwrap()));
}

#[cfg(feature = "netcode")]
pub fn join_netcode_server(
    mut commands: Commands,
    mut join_lobby: EventReader<JoinNetcodeLobby>,
    #[cfg(prod)] mut ui: ResMut<NextState<MainMenuStates>>,
) {
    let server_addr = match join_lobby.read().next() {
        Some(value) => value.0,
        None => return,
    };

    use bevy_renet::renet::transport::{ClientAuthentication, NetcodeClientTransport};
    use shared::networking::{PlayerCommand, PROTOCOL_ID};
    use std::{net::UdpSocket, time::SystemTime};

    let client = RenetClient::new(connection_config());

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

    match NetcodeClientTransport::new(current_time, authentication, socket) {
        Ok(transport) => {
            commands.insert_resource(client);
            commands.insert_resource(transport);
            commands.insert_resource(CurrentClientId(ClientId::from_raw(client_id)));
            #[cfg(prod)]
            ui.set(MainMenuStates::Lobby);
        }
        Err(error) => println!("join_netcode_server error {}", error),
    }
}
