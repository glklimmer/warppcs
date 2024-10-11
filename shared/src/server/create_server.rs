use bevy::prelude::*;

use crate::{networking::connection_config, steamworks::SteamworksClient};
use bevy_renet::renet::RenetServer;

pub fn create_server(mut commands: Commands) {
    use renet_steam::bevy::{SteamServerConfig, SteamServerTransport};
    use renet_steam::AccessPermission;

    let server: RenetServer = RenetServer::new(connection_config());
    commands.insert_resource(server);

    commands.add(|world: &mut World| {
        let steam_client = world.get_resource::<SteamworksClient>().unwrap();
        println!("From Server lib: {}", steam_client.friends().name());
        let steam_transport_config = SteamServerConfig {
            max_clients: 10,
            access_permission: AccessPermission::Public,
        };

        world.insert_non_send_resource(
            SteamServerTransport::new(steam_client, steam_transport_config).unwrap(),
        );
    });
}
