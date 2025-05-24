use aeronet::{
    io::{Session, SessionEndpoint, connection::Disconnected},
    transport::TransportConfig,
};
use aeronet_replicon::client::{AeronetRepliconClient, AeronetRepliconClientPlugin};
use aeronet_webtransport::{
    cert,
    client::{WebTransportClient, WebTransportClientPlugin},
};
use bevy::prelude::*;

use shared::server::create_server::WEB_TRANSPORT_PORT;

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
            commands.insert_resource(LocalPlayer::new(steam_client.user().steam_id().raw()));
            ui.set(MainMenuStates::Lobby);
        }
        Err(error) => println!("join_netcode_server error {}", error),
    }
}

// #[cfg(feature = "netcode")]
// // pub fn join_netcode_server(mut commands: Commands, channels: Res<RepliconChannels>) {
// //     use shared::networking::PROTOCOL_ID;
// //     use std::{net::UdpSocket, time::SystemTime};

// //     let server_channels_config = channels.server_configs();
// //     let client_channels_config = channels.client_configs();

// //     let client = RenetClient::new(ConnectionConfig {
// //         server_channels_config,
// //         client_channels_config,
// //         ..Default::default()
// //     });

// //     let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
// //     let current_time = SystemTime::now()
// //         .duration_since(SystemTime::UNIX_EPOCH)
// //         .unwrap();
// //     let client_id = current_time.as_millis() as u64;
// //     let authentication = ClientAuthentication::Unsecure {
// //         client_id,
// //         protocol_id: PROTOCOL_ID,
// //         server_addr: "127.0.0.1:5000".parse().unwrap(),
// //         user_data: None,
// //     };

// //     match NetcodeClientTransport::new(current_time, authentication, socket) {
// //         Ok(transport) => {
// //             commands.insert_resource(client);
// //             commands.insert_resource(transport);
// //             info!("Successfully joined server.");
// //         }
// //         Err(error) => println!("join_netcode_server error {}", error),
// //     }
// // }

#[derive(Event)]
pub struct JoinWebTransportServer;

pub struct JoinServerPlugin;

impl Plugin for JoinServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WebTransportClientPlugin, AeronetRepliconClientPlugin));

        app.add_event::<JoinWebTransportServer>();
        app.add_observer(join_web_transport_server)
            .add_observer(on_connecting)
            .add_observer(on_connected)
            .add_observer(on_disconnected);
    }
}

fn join_web_transport_server(_: Trigger<JoinWebTransportServer>, mut commands: Commands) {
    let config = web_transport_config("".to_string());
    let default_target = format!("https://127.0.0.1:{WEB_TRANSPORT_PORT}");
    commands
        .spawn(Name::new("client"))
        .queue(WebTransportClient::connect(config, default_target));
}

type WebTransportClientConfig = aeronet_webtransport::client::ClientConfig;

fn web_transport_config(cert_hash: String) -> WebTransportClientConfig {
    use {aeronet_webtransport::wtransport::tls::Sha256Digest, core::time::Duration};

    let config = WebTransportClientConfig::builder().with_bind_default();

    let config = if cert_hash.is_empty() {
        warn!("Connecting without certificate validation");
        config.with_no_cert_validation()
    } else {
        match cert::hash_from_b64(&cert_hash) {
            Ok(hash) => config.with_server_certificate_hashes([Sha256Digest::new(hash)]),
            Err(err) => {
                warn!("Failed to read certificate hash from string: {err:?}");
                config.with_server_certificate_hashes([])
            }
        }
    };

    config
        .keep_alive_interval(Some(Duration::from_secs(1)))
        .max_idle_timeout(Some(Duration::from_secs(5)))
        .expect("should be a valid idle timeout")
        .build()
}

fn on_connecting(
    trigger: Trigger<OnAdd, SessionEndpoint>,
    names: Query<&Name>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let name = names
        .get(entity)
        .expect("our session entity should have a name");
    info!("Entity {} in SessionEndpoint {}", entity, name);

    // IMPORTANT
    //
    // Make sure to insert this component into your client entity,
    // so that `aeronet_replicon` knows you want to use this for `bevy_replicon`!
    //
    // You can also do this when `spawn`ing the entity instead, which is a bit more
    // efficient. We just do it on `OnAdd, SessionEndpoint`, since we have
    // multiple `spawn` calls, and it's nicer to centralize inserting this
    // component in a single place.
    commands.entity(entity).insert(AeronetRepliconClient);
}

fn on_connected(trigger: Trigger<OnAdd, Session>, mut commands: Commands) {
    let entity = trigger.target();
    info!("Entity {} in Session", entity);

    commands
        .entity(entity)
        .insert((TransportConfig { ..default() },));
}

fn on_disconnected(trigger: Trigger<Disconnected>, names: Query<&Name>) {
    let session = trigger.target();
    let name = names
        .get(session)
        .expect("our session entity should have a name");
    match &*trigger {
        Disconnected::ByUser(reason) => {
            format!("{name} disconnected by user: {reason}")
        }
        Disconnected::ByPeer(reason) => {
            format!("{name} disconnected by peer: {reason}")
        }
        Disconnected::ByError(err) => {
            format!("{name} disconnected due to error: {err:?}")
        }
    };
}
