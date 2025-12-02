use bevy::prelude::*;

use aeronet::{
    io::{Session, SessionEndpoint, connection::Disconnected},
    transport::TransportConfig,
};
use aeronet_replicon::client::{AeronetRepliconClient, AeronetRepliconClientPlugin};

pub struct JoinServerPlugin;

impl Plugin for JoinServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AeronetRepliconClientPlugin)
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
    use shared::server::create_server::WEB_TRANSPORT_PORT;

    let config = web_transport_config(None);
    let default_target = format!("https://127.0.0.1:{WEB_TRANSPORT_PORT}");

    commands
        .spawn_empty()
        .queue(WebTransportClient::connect(config, default_target));
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
    use bevy_steamworks::{ClientManager, GameLobbyJoinRequested};

    if let Some(&SteamworksEvent::GameLobbyJoinRequested(GameLobbyJoinRequested {
        friend_steam_id,
        ..
    })) = join_lobby.read().next()
    {
        commands
            .spawn_empty()
            .queue(SteamNetClient::<ClientManager>::connect(
                SessionConfig::default(),
                friend_steam_id,
            ));
    }
}

fn on_connecting(trigger: On<Add, SessionEndpoint>, mut commands: Commands) {
    let entity = trigger.target();

    info!("Joining server...");

    commands.entity(entity).insert(AeronetRepliconClient);
}

fn on_connected(trigger: On<Add, Session>, mut commands: Commands) {
    let entity = trigger.target();

    info!("Joined server.");

    commands
        .entity(entity)
        .insert((TransportConfig { ..default() },));
}

fn on_disconnected(trigger: On<Disconnected>) {
    match &*trigger {
        Disconnected::ByUser(reason) => {
            format!("Disconnected by user: {reason}")
        }
        Disconnected::ByPeer(reason) => {
            format!("Disconnected by peer: {reason}")
        }
        Disconnected::ByError(err) => {
            format!("Disconnected due to error: {err:?}")
        }
    };
}
