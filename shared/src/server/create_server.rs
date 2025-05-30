use std::time::Duration;

use aeronet::io::{Session, SessionEndpoint, connection::Disconnected, server::Server};
use aeronet_replicon::server::{AeronetRepliconServer, AeronetRepliconServerPlugin};
use aeronet_steam::{
    SessionConfig, SteamworksClient,
    server::{ListenTarget, SteamNetServer},
};
use aeronet_webtransport::{
    server::{SessionRequest, SessionResponse, WebTransportServer, WebTransportServerPlugin},
    wtransport,
};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use steamworks::ClientManager;

use crate::{ClientPlayerMap, Player, SetLocalPlayer};

pub const WEB_TRANSPORT_PORT: u16 = 25571;

pub struct CreateServerPlugin;

impl Plugin for CreateServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WebTransportServerPlugin, AeronetRepliconServerPlugin));

        app.add_observer(on_session_request_steam)
            .add_observer(on_session_request)
            .add_observer(on_created)
            .add_observer(on_connecting)
            .add_observer(on_connected)
            .add_observer(on_disconnected);
    }
}

pub fn create_web_transport_server(mut commands: Commands) {
    let identity = wtransport::Identity::self_signed(["localhost", "127.0.0.1", "::1"])
        .expect("all given SANs should be valid DNS names");
    let config = web_transport_config(identity);

    commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            AeronetRepliconServer,
        ))
        .queue(WebTransportServer::open(config));

    info!("Creating server...")
}

type WebTransportServerConfig = aeronet_webtransport::server::ServerConfig;

fn web_transport_config(identity: wtransport::Identity) -> WebTransportServerConfig {
    WebTransportServerConfig::builder()
        .with_bind_default(WEB_TRANSPORT_PORT)
        .with_identity(identity)
        .keep_alive_interval(Some(Duration::from_secs(1)))
        .max_idle_timeout(Some(Duration::from_secs(5)))
        .expect("should be a valid idle timeout")
        .build()
}

pub fn create_steam_server(mut commands: Commands, client: Res<SteamworksClient>) {
    let target = ListenTarget::Peer { virtual_port: 0 };

    client
        .matchmaking()
        .create_lobby(steamworks::LobbyType::FriendsOnly, 8, |result| {
            let Ok(lobby_id) = result else {
                error!("Could not create steam lobby.");
                return;
            };

            info!("Created steam lobby: {:?}", lobby_id);
        });

    commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            AeronetRepliconServer,
        ))
        .queue(SteamNetServer::<ClientManager>::open(
            SessionConfig::default(),
            target,
        ));

    info!("Creating server...")
}

fn on_created(
    _: Trigger<OnAdd, Server>,
    mut client_player_map: ResMut<ClientPlayerMap>,
    mut commands: Commands,
) {
    info!("Successfully created server");

    let server_player = commands.spawn(Player).id();

    client_player_map.insert(SERVER, server_player);

    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        event: SetLocalPlayer(server_player),
    });
}

fn on_session_request_steam(mut request: Trigger<aeronet_steam::server::SessionRequest>) {
    let client = request.steam_id;
    info!("Steamclient {:?} requesting connection...", client);

    request.respond(aeronet_steam::server::SessionResponse::Accepted);
}

fn on_session_request(mut request: Trigger<SessionRequest>) {
    let client = request.target();
    info!("Client {client} requesting connection...");
    request.respond(SessionResponse::Accepted);
}

fn on_connecting(trigger: Trigger<OnAdd, SessionEndpoint>) {
    let client = trigger.target();
    info!("Client {client} connecting...");
}

fn on_connected(trigger: Trigger<OnAdd, Session>) {
    let client = trigger.target();
    info!("Client {client} connected.");
}

fn on_disconnected(trigger: Trigger<Disconnected>) {
    let client = trigger.target();

    match &*trigger {
        Disconnected::ByUser(reason) => {
            info!("Client {client} disconnected from server by user: {reason}");
        }
        Disconnected::ByPeer(reason) => {
            info!("Client {client} disconnected from server by peer: {reason}");
        }
        Disconnected::ByError(err) => {
            warn!("Client {client} disconnected from server due to error: {err:?}");
        }
    }
}
