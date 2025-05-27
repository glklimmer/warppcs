use std::time::Duration;

use aeronet::io::{
    Session,
    connection::{Disconnected, LocalAddr},
    server::Server,
};
use aeronet_replicon::server::{AeronetRepliconServer, AeronetRepliconServerPlugin};
use aeronet_webtransport::{
    cert,
    server::{SessionRequest, SessionResponse, WebTransportServer, WebTransportServerPlugin},
    wtransport,
};
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::{ClientPlayerMap, Player, SetLocalPlayer};

pub const WEB_TRANSPORT_PORT: u16 = 25571;

// pub fn create_steam_server(mut commands: Commands, channels: Res<RepliconChannels>) {
//     use crate::steamworks::SteamworksClient;
//     use renet_steam::AccessPermission;
//     use renet_steam::SteamServerConfig;
//     use renet_steam::SteamServerTransport;

//     let server_channels_config = channels.server_configs();
//     let client_channels_config = channels.client_configs();

//     let server = RenetServer::new(ConnectionConfig {
//         server_channels_config,
//         client_channels_config,
//         ..Default::default()
//     });

//     commands.insert_resource(server);

//     commands.queue(|world: &mut World| {
//         let steam_client = world.get_resource::<SteamworksClient>().unwrap();
//         println!("From Server lib: {}", steam_client.friends().name());
//         let steam_transport_config = SteamServerConfig {
//             max_clients: 10,
//             access_permission: AccessPermission::Public,
//         };

//         world.insert_non_send_resource(
//             SteamServerTransport::new(steam_client, steam_transport_config).unwrap(),
//         );
//     });
// }

// pub fn create_netcode_server(
//     mut commands: Commands,
//     channels: Res<RepliconChannels>,
//     mut set_local_player: EventWriter<ToClients<SetLocalPlayer>>,
//     mut client_player_map: ResMut<ClientPlayerMap>,
// ) {
//     use crate::networking::PROTOCOL_ID;
//     use std::{net::UdpSocket, time::SystemTime};

//     let server_channels_config = channels.server_configs();
//     let client_channels_config = channels.client_configs();

//     let server = RenetServer::new(ConnectionConfig {
//         server_channels_config,
//         client_channels_config,
//         ..Default::default()
//     });

//     let public_addr = "127.0.0.1:5000".parse().unwrap();
//     let socket = UdpSocket::bind(public_addr).unwrap();
//     let current_time: std::time::Duration = SystemTime::now()
//         .duration_since(SystemTime::UNIX_EPOCH)
//         .unwrap();
//     let server_config = ServerConfig {
//         current_time,
//         max_clients: 64,
//         protocol_id: PROTOCOL_ID,
//         public_addresses: vec![public_addr],
//         authentication: ServerAuthentication::Unsecure,
//     };

//     let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
//     commands.insert_resource(server);
//     commands.insert_resource(transport);

//     let player = commands.spawn(Player).id();

//     client_player_map.insert(SERVER, player);

//     set_local_player.write(ToClients {
//         mode: SendMode::Broadcast,
//         event: SetLocalPlayer(player),
//     });

//     info!("Successfully started server.")
// }
//
//
//

pub struct CreateServerPlugin;

impl Plugin for CreateServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WebTransportServerPlugin, AeronetRepliconServerPlugin));

        app.add_observer(on_session_request)
            .add_observer(on_opened)
            .add_observer(on_connected)
            .add_observer(on_disconnected);
    }
}

pub fn create_web_transport_server(mut commands: Commands) {
    let identity = wtransport::Identity::self_signed(["localhost", "127.0.0.1", "::1"])
        .expect("all given SANs should be valid DNS names");
    let cert = &identity.certificate_chain().as_slice()[0];
    let spki_fingerprint = cert::spki_fingerprint_b64(cert).expect("should be a valid certificate");
    let cert_hash = cert::hash_to_b64(cert.hash());
    info!("************************");
    info!("SPKI FINGERPRINT");
    info!("  {spki_fingerprint}");
    info!("CERTIFICATE HASH");
    info!("  {cert_hash}");
    info!("************************");

    let config = web_transport_config(identity);
    let server = commands
        .spawn((
            Name::new("WebTransport Server"),
            Transform::default(),
            Visibility::default(),
            // IMPORTANT
            //
            // Make sure to insert this component into your server entity,
            // so that `aeronet_replicon` knows you want to use this for `bevy_replicon`!
            AeronetRepliconServer,
        ))
        .queue(WebTransportServer::open(config))
        .id();

    info!("Opening WebTransport server {server}");
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

fn on_session_request(mut request: Trigger<SessionRequest>, clients: Query<&ChildOf>) {
    let client = request.target();
    let Ok(&ChildOf(server)) = clients.get(client) else {
        return;
    };

    info!("{client} connecting to {server} with headers:");
    for (header_key, header_value) in &request.headers {
        info!("  {header_key}: {header_value}");
    }

    request.respond(SessionResponse::Accepted);
}

fn on_opened(
    trigger: Trigger<OnAdd, Server>,
    servers: Query<&LocalAddr>,
    mut client_player_map: ResMut<ClientPlayerMap>,
    mut commands: Commands,
) {
    let server = trigger.target();
    let local_addr = servers
        .get(server)
        .expect("opened server should have a binding socket `LocalAddr`");
    info!("{server} opened on {}", **local_addr);

    let server_player = commands.spawn(Player).id();

    client_player_map.insert(SERVER, server_player);

    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        event: SetLocalPlayer(server_player),
    });
}

fn on_connected(trigger: Trigger<OnAdd, Session>, clients: Query<&ChildOf>) {
    let client = trigger.target();
    let Ok(&ChildOf(server)) = clients.get(client) else {
        return;
    };
    info!("{client} connected to {server}");
}

fn on_disconnected(trigger: Trigger<Disconnected>, clients: Query<&ChildOf>) {
    let client = trigger.target();
    let Ok(&ChildOf(server)) = clients.get(client) else {
        return;
    };

    match &*trigger {
        Disconnected::ByUser(reason) => {
            info!("{client} disconnected from {server} by user: {reason}");
        }
        Disconnected::ByPeer(reason) => {
            info!("{client} disconnected from {server} by peer: {reason}");
        }
        Disconnected::ByError(err) => {
            warn!("{client} disconnected from {server} due to error: {err:?}");
        }
    }
}
