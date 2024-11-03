use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_renet::renet::{transport::NetcodeServerTransport, ClientId, RenetServer};

use crate::networking::{Checkbox, MultiplayerRoles, ServerChannel, ServerMessages};

#[derive(Event)]
pub struct PlayerJoinedLobby(pub ClientId);

#[derive(Event)]
pub struct PlayerLeavedLobby(pub ClientId);

#[derive(Event)]
pub struct PlayerChangedReady {
    pub id: ClientId,
    pub ready_state: Checkbox,
}

#[derive(Default, Resource)]
pub struct GameLobby {
    pub players: BTreeMap<ClientId, Checkbox>,
}

impl GameLobby {
    pub fn all_ready(&self) -> bool {
        self.players
            .values()
            .all(|ready| ready.eq(&Checkbox::Checked))
    }
}

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameLobby>();
        app.add_event::<PlayerJoinedLobby>();
        app.add_event::<PlayerLeavedLobby>();
        app.add_event::<PlayerChangedReady>();

        app.add_systems(
            FixedUpdate,
            lobby_check.run_if(on_event::<PlayerChangedReady>()),
        );

        app.add_systems(
            FixedUpdate,
            update_lobby.run_if(resource_exists::<RenetServer>),
        );

        app.add_systems(
            FixedUpdate,
            remove_player.run_if(on_event::<PlayerLeavedLobby>()),
        );
    }
}

fn update_lobby(
    mut server: ResMut<RenetServer>,
    mut game_lobby: ResMut<GameLobby>,
    mut player_joined: EventReader<PlayerJoinedLobby>,
) {
    for new_player in player_joined.read() {
        println!("Server Lobby id {}", new_player.0);
        // Update all Players for new one
        for player in &mut game_lobby.players.iter() {
            let message = bincode::serialize(&ServerMessages::PlayerJoinedLobby {
                id: new_player.0,
                ready_state: Checkbox::Unchecked,
            })
            .unwrap();

            server.send_message(*player.0, ServerChannel::ServerMessages, message);
        }

        game_lobby.players.insert(new_player.0, Checkbox::Unchecked);

        for player in &mut game_lobby.players.iter() {
            let message = bincode::serialize(&ServerMessages::PlayerJoinedLobby {
                id: *player.0,
                ready_state: player.1.clone(),
            })
            .unwrap();

            server.send_message(new_player.0, ServerChannel::ServerMessages, message);
        }
    }
}

fn lobby_check(
    mut server: ResMut<RenetServer>,
    mut game_lobby: ResMut<GameLobby>,
    mut player_checkbox: EventReader<PlayerChangedReady>,
) {
    for player in player_checkbox.read() {
        game_lobby
            .players
            .insert(player.id, player.ready_state.clone());

        let message = ServerMessages::LobbyPlayerReadyState {
            id: player.id,
            ready_state: player.ready_state.clone(),
        };

        let player_state_message = bincode::serialize(&message).unwrap();

        server.broadcast_message(ServerChannel::ServerMessages, player_state_message);
    }
}

fn remove_player(
    mut commands: Commands,
    mut ready_players: ResMut<GameLobby>,
    mut player_left: EventReader<PlayerLeavedLobby>,
    mut server: ResMut<RenetServer>,
    mut multiplayer_roles: ResMut<NextState<MultiplayerRoles>>,
) {
    for client_id in player_left.read() {
        let host_player = ready_players.players.keys().next().copied().unwrap();
        match ready_players.players.remove_entry(&client_id.0) {
            Some(removed_player) => {
                if removed_player.0.eq(&host_player) {
                    server.disconnect_all();
                    commands.remove_resource::<RenetServer>();
                    commands.remove_resource::<NetcodeServerTransport>();
                    multiplayer_roles.set(MultiplayerRoles::NotInGame);
                } else {
                    let message = ServerMessages::PlayerLeftLobby { id: client_id.0 };

                    let player_state_message = bincode::serialize(&message).unwrap();

                    server.broadcast_message(ServerChannel::ServerMessages, player_state_message);
                }
            }
            None => println!("Client ID: {} not in Lobby", client_id.0),
        }
    }
}
