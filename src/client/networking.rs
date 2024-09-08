use bevy::prelude::*;

use crate::{
    client::animation::{AnimationIndices, Animations, AnimationsState, CurrentAnimation},
    shared::networking::{
        connection_config, ClientChannel, NetworkedEntities, PlayerCommand, PlayerInput,
        ServerChannel, ServerMessages, PROTOCOL_ID,
    },
};
use bevy_renet::{
    client_connected,
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
        ClientId, RenetClient,
    },
    RenetClientPlugin,
};
use std::{
    collections::HashMap,
    net::UdpSocket,
    time::{Duration, SystemTime},
};

pub struct ClientNetworkingPlugin;

impl Plugin for ClientNetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);

        add_netcode_network(app);

        app.add_event::<UnitEvent>();

        app.insert_resource(NetworkMapping::default());
        app.insert_resource(ClientLobby::default());

        app.add_systems(
            Update,
            (
                client_sync_players,
                client_send_input,
                client_send_player_commands,
            )
                .in_set(Connected),
        );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

#[derive(Debug, Default, Resource)]
struct ClientLobby {
    players: HashMap<ClientId, PlayerEntityMapping>,
}

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Debug, Resource)]
struct CurrentClientId(u64);

#[derive(Debug)]
struct PlayerEntityMapping {
    client_entity: Entity,
    server_entity: Entity,
}

#[derive(Default, Resource)]
struct NetworkMapping(HashMap<Entity, Entity>);

#[derive(Event)]
pub enum UnitEvent {
    MeleeAttack(Entity),
}

fn add_netcode_network(app: &mut App) {
    app.add_plugins(bevy_renet::transport::NetcodeClientPlugin);

    app.configure_sets(Update, Connected.run_if(client_connected));

    let client = RenetClient::new(connection_config());

    let server_addr = "127.0.0.1:6969".parse().unwrap();
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

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    app.insert_resource(client);
    app.insert_resource(transport);
    app.insert_resource(CurrentClientId(client_id));

    // If any error is found we just panic
    #[allow(clippy::never_loop)]
    fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
        for e in renet_error.read() {
            panic!("{}", e);
        }
    }

    app.add_systems(Update, panic_on_error_system);
}

fn client_sync_players(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    client_id: Res<CurrentClientId>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut unit_events: EventWriter<UnitEvent>,
) {
    let client_id = client_id.0;
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerCreate {
                id,
                translation,
                entity,
                color,
            } => {
                let texture = asset_server.load("f1_general.png");
                let layout_walk = TextureAtlasLayout::from_grid(
                    UVec2::splat(100),
                    1,
                    8,
                    Some(UVec2::new(1, 1)),
                    None,
                );

                let layout_idle = TextureAtlasLayout::from_grid(
                    UVec2::splat(100),
                    1,
                    8,
                    Some(UVec2::new(1, 1)),
                    Some(UVec2::new(100, 1)),
                );

                let layout_attack = TextureAtlasLayout::from_grid(
                    UVec2::splat(100),
                    1,
                    8,
                    Some(UVec2::new(1, 1)),
                    Some(UVec2::new(700, 1)),
                );

                let idle_id = texture_atlas_layouts.add(layout_idle);
                let walk_id = texture_atlas_layouts.add(layout_walk);
                let attack_id = texture_atlas_layouts.add(layout_attack);

                println!("Player {} connected.", id);
                let mut client_entity = commands.spawn((
                    SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(translation[0], translation[1], translation[2]),
                            scale: Vec3::splat(2.0),
                            ..Default::default()
                        },
                        texture,
                        ..default()
                    },
                    Animations {
                        idle: (
                            idle_id.clone(),
                            Timer::from_seconds(0.1, TimerMode::Repeating),
                        ),
                        walk: (walk_id, Timer::from_seconds(0.08, TimerMode::Repeating)),
                        attack: (attack_id, Timer::from_seconds(0.05, TimerMode::Repeating)),
                    },
                    TextureAtlas {
                        layout: idle_id,
                        index: 7,
                    },
                    AnimationIndices { first: 7, last: 0 },
                    CurrentAnimation {
                        state: AnimationsState::Idle,
                        frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                        animation_duration: Timer::from_seconds(0., TimerMode::Once),
                    },
                ));

                if client_id == id.raw() {
                    client_entity.insert(ControlledPlayer);
                }

                let player_info = PlayerEntityMapping {
                    server_entity: entity,
                    client_entity: client_entity.id(),
                };
                lobby.players.insert(id, player_info);
                network_mapping.0.insert(entity, client_entity.id());
            }
            ServerMessages::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                if let Some(PlayerEntityMapping {
                    server_entity,
                    client_entity,
                }) = lobby.players.remove(&id)
                {
                    commands.entity(client_entity).despawn();
                    network_mapping.0.remove(&server_entity);
                }
            }
            ServerMessages::MeleeAttack { entity } => {
                if let Some(entity) = network_mapping.0.get(&entity) {
                    unit_events.send(UnitEvent::MeleeAttack(*entity));
                }
            }
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();

        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping
                .0
                .get(&networked_entities.entities[i].entity)
            {
                let network_entity = &networked_entities.entities[i];
                let movement = network_entity.movement.clone();

                commands.entity(*entity).insert(movement);
            }
        }
    }
}

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();
    client.send_message(ClientChannel::Input, input_message);
}

fn client_send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.read() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command, command_message);
    }
}
