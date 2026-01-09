use bevy::prelude::*;

use aeronet::{
    io::{
        Session, SessionEndpoint,
        connection::{DisconnectReason, Disconnected},
    },
    transport::TransportConfig,
};
use aeronet_replicon::client::{AeronetRepliconClient, AeronetRepliconClientPlugin};
use bevy::utils::default;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum ClientState {
    #[default]
    Offline,
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Resource, Clone)]
enum LastConnection {
    #[cfg(feature = "netcode")]
    Web(String),
    #[cfg(feature = "steam")]
    Steam(bevy_steamworks::SteamId),
}

#[derive(Resource)]
struct ReconnectTimer(Timer);

pub struct JoinServerPlugin;

impl Plugin for JoinServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AeronetRepliconClientPlugin)
            .insert_state(ClientState::default())
            .insert_resource(ReconnectTimer(Timer::from_seconds(
                5.0,
                TimerMode::Repeating,
            )))
            .add_systems(
                Update,
                reconnect_system.run_if(in_state(ClientState::Disconnected)),
            )
            .add_observer(on_connecting)
            .add_observer(on_connected)
            .add_observer(on_disconnected);

        #[cfg(feature = "netcode")]
        {
            use aeronet_webtransport::client::WebTransportClientPlugin;
            app.add_plugins(WebTransportClientPlugin);
        }

        #[cfg(feature = "steam")]
        {
            app.add_systems(
                Update,
                join_steam_server.run_if(on_message::<SteamworksEvent>),
            );
        }
    }
}

#[cfg(feature = "netcode")]
pub fn join_web_transport_server(mut commands: Commands) {
    use aeronet_webtransport::client::WebTransportClient;
    use lobby::create_server::WEB_TRANSPORT_PORT;

    let config = web_transport_config(None);
    let default_target = format!("https://127.0.0.1:{WEB_TRANSPORT_PORT}");

    commands
        .spawn_empty()
        .queue(WebTransportClient::connect(config, default_target.clone()));

    commands.insert_resource(LastConnection::Web(default_target));
}

#[cfg(feature = "netcode")]
type WebTransportClientConfig = aeronet_webtransport::client::ClientConfig;

#[cfg(feature = "netcode")]
fn web_transport_config(cert_hash: Option<String>) -> WebTransportClientConfig {
    use aeronet_webtransport::{cert::hash_from_b64, wtransport::tls::Sha256Digest};
    use std::time::Duration;

    let config = WebTransportClientConfig::builder().with_bind_default();

    let config = if let Some(hash) = cert_hash {
        match hash_from_b64(&hash) {
            Ok(hash) => config.with_server_certificate_hashes([Sha256Digest::new(hash)]),
            Err(err) => {
                warn!("Failed to read certificate hash from string: {err:?}");
                config.with_server_certificate_hashes([])
            }
        }
    } else {
        config.with_no_cert_validation()
    };

    config
        .keep_alive_interval(Some(Duration::from_secs(1)))
        .max_idle_timeout(Some(Duration::from_secs(5)))
        .expect("should be a valid idle timeout")
        .build()
}

#[cfg(feature = "steam")]
use bevy_steamworks::SteamworksEvent;

#[cfg(feature = "steam")]
fn join_steam_server(mut join_lobby: MessageReader<SteamworksEvent>, mut commands: Commands) {
    use aeronet_steam::SessionConfig;
    use aeronet_steam::client::SteamNetClient;
    use bevy_steamworks::{CallbackResult, GameLobbyJoinRequested};

    let maybe_event = join_lobby.read().next();
    let Some(SteamworksEvent::CallbackResult(result)) = maybe_event else {
        return;
    };
    let CallbackResult::GameLobbyJoinRequested(GameLobbyJoinRequested {
        friend_steam_id, ..
    }) = result
    else {
        return;
    };

    commands.spawn_empty().queue(SteamNetClient::connect(
        SessionConfig::default(),
        *friend_steam_id,
    ));

    commands.insert_resource(LastConnection::Steam(*friend_steam_id));
}

fn reconnect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ReconnectTimer>,
    last_connection: Option<Res<LastConnection>>,
    mut client_state: ResMut<NextState<ClientState>>,
) {
    if let Some(last_connection) = last_connection {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            info!("Attempting to reconnect...");
            match &*last_connection {
                #[cfg(feature = "netcode")]
                LastConnection::Web(addr) => {
                    use aeronet_webtransport::client::WebTransportClient;
                    let config = web_transport_config(None);
                    commands
                        .spawn_empty()
                        .queue(WebTransportClient::connect(config, addr.clone()));
                }
                #[cfg(feature = "steam")]
                LastConnection::Steam(id) => {
                    use aeronet_steam::{SessionConfig, client::SteamNetClient};
                    commands
                        .spawn_empty()
                        .queue(SteamNetClient::connect(SessionConfig::default(), *id));
                }
            }
            client_state.set(ClientState::Connecting);
        }
    }
}

fn on_connecting(
    trigger: On<Add, SessionEndpoint>,
    mut commands: Commands,
    mut client_state: ResMut<NextState<ClientState>>,
) {
    let entity = trigger.entity;

    info!("Joining server...");

    commands.entity(entity).insert(AeronetRepliconClient);
    client_state.set(ClientState::Connecting);
}

fn on_connected(
    trigger: On<Add, Session>,
    mut commands: Commands,
    mut client_state: ResMut<NextState<ClientState>>,
) {
    let entity = trigger.entity;

    info!("Joined server.");

    commands
        .entity(entity)
        .insert((TransportConfig { ..default() },));
    client_state.set(ClientState::Connected);
}

fn on_disconnected(trigger: On<Disconnected>, mut client_state: ResMut<NextState<ClientState>>) {
    match &trigger.reason {
        DisconnectReason::ByUser(reason) => {
            info!("Disconnected by user: {reason}");
        }
        DisconnectReason::ByPeer(reason) => {
            info!("Disconnected by peer: {reason}");
        }
        DisconnectReason::ByError(err) => {
            info!("Disconnected due to error: {err:?}");
        }
    };
    client_state.set(ClientState::Disconnected);
}
