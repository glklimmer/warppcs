use bevy::prelude::*;

use bevy_renet::renet::RenetClient;
use renet_steam::SteamClientTransport;
use shared::{
    networking::{connection_config, GameState},
    steamworks::SteamworksClient,
};

use crate::{networking::CurrentClientId, ui::JoinLobbyRequest};

pub fn join_server(
    mut commands: Commands,
    steam_client: Res<SteamworksClient>,
    mut server_steam_id: EventReader<JoinLobbyRequest>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let client = RenetClient::new(connection_config());

    steam_client.networking_utils().init_relay_network_access();

    println!("From Client {}", steam_client.friends().name());

    let server_steam_id = match server_steam_id.read().next() {
        Some(value) => value.0,
        None => steam_client.user().steam_id(),
    };

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

    next_state.set(GameState::JoinLobbyHost);
}
