use bevy::prelude::*;

use bevy_renet::renet::RenetClient;
use renet_steam::SteamClientTransport;
use shared::{networking::connection_config, steamworks::SteamworksClient};

use crate::{
    networking::CurrentClientId,
    ui::{JoinLobbyRequest, MainMenuStates},
};

pub fn join_own_server(
    mut join_lobby: EventWriter<JoinLobbyRequest>,
    steam_client: Res<SteamworksClient>,
) {
    join_lobby.send(JoinLobbyRequest(steam_client.user().steam_id()));
}

pub fn join_server(
    mut commands: Commands,
    steam_client: Res<SteamworksClient>,
    mut join_lobby: EventReader<JoinLobbyRequest>,
    mut next_state: ResMut<NextState<MainMenuStates>>,
) {
    let server_steam_id = match join_lobby.read().next() {
        Some(value) => value.0,
        None => return,
    };

    let client = RenetClient::new(connection_config());

    steam_client.networking_utils().init_relay_network_access();

    println!("From Client {}", steam_client.friends().name());

    let transport = SteamClientTransport::new(&steam_client, &server_steam_id);
    let transport = match transport {
        Ok(transport) => transport,
        Err(e) => {
            println!("Id {:?}", server_steam_id);
            panic!("Error when trying to create SteamClientTransport: {}", e)
        }
    };

    commands.insert_resource(transport);
    commands.insert_resource(client);
    commands.insert_resource(CurrentClientId(steam_client.user().steam_id().raw()));

    next_state.set(MainMenuStates::Lobby);
}
