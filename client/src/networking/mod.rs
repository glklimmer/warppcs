use bevy::ecs::system::SystemParam;
use bevy::{color::palettes::css::YELLOW, prelude::*};

use bevy::sprite::Mesh2dHandle;
use bevy_renet::client_just_disconnected;
use bevy_renet::{
    renet::{ClientId, RenetClient},
    RenetClientPlugin,
};
use shared::networking::{MultiplayerRoles, SpawnFlag};
use shared::GameState;
use shared::{
    map::{base::BaseScene, GameSceneType},
    networking::{
        ClientChannel, NetworkedEntities, PlayerCommand, PlayerInput, Rotation, ServerChannel,
        ServerMessages, SpawnPlayer, SpawnProjectile, SpawnUnit,
    },
};
use spawn::SpawnPlugin;
use std::collections::HashMap;

use crate::ui::{MainMenuStates, PlayerCheckbox, PlayerJoinedLobby, PlayerLeftLobby};

pub mod join_server;
mod spawn;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Connected;

#[derive(Debug, Default, Resource)]
pub struct ClientPlayers {
    pub players: HashMap<ClientId, PlayerEntityMapping>,
}

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Debug, Resource)]
pub struct CurrentClientId(pub ClientId);

#[derive(Debug)]
pub struct PlayerEntityMapping {
    pub client_entity: Entity,
    pub server_entity: Entity,
}

#[derive(Default, Resource)]
struct NetworkMapping(HashMap<Entity, Entity>);

pub enum Change {
    Rotation(Rotation),
    Movement(bool),
    Attack,
}

#[derive(Event)]
pub struct NetworkEvent {
    pub entity: Entity,
    pub change: Change,
}

#[derive(Component)]
struct PartOfScene;

pub struct ClientNetworkPlugin;

impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);
        app.add_plugins(SpawnPlugin);

        app.insert_resource(NetworkMapping::default());
        app.insert_resource(ClientPlayers::default());

        app.add_event::<NetworkEvent>();

        app.add_systems(
            Update,
            (
                client_sync_players,
                client_send_input,
                client_send_player_commands,
            )
                .in_set(Connected),
        );

        app.add_systems(Update, disconnect_client.run_if(client_just_disconnected));
    }
}

fn disconnect_client(
    mut menu_state: ResMut<NextState<MainMenuStates>>,
    mut multiplayer_roles: ResMut<NextState<MultiplayerRoles>>,
) {
    println!("Disconnecting");
    menu_state.set(MainMenuStates::Multiplayer);
    multiplayer_roles.set(MultiplayerRoles::NotInGame);
}

// fn listen_for_game_invites(steam_client: Res<SteamworksClient>, mut commands: Commands) {
//     let friends = steam_client.friends();

//     for friend in friends.get_friends(FriendFlags::IMMEDIATE) {
//         if let Some(game_invite) = friend.game_played() {
//             if game_invite.lobby_id.is_some() {
//                 println!("Received game invite from: {}", friend.name());
//                 // Handle the invite here, e.g., join the lobby or show a notification
//                 // You might want to spawn an event or entity to handle this in your game logic
//                 commands.spawn(GameInviteEvent {
//                     friend_name: friend.name().to_string(),
//                     lobby_id: game_invite.lobby_id.unwrap(),
//                 });
//             }
//         }
//     }
//}
#[derive(SystemParam)]
pub struct ClientSyncPlayersParams<'w, 's> {
    commands: Commands<'w, 's>,
    transforms: Query<'w, 's, &'static mut Transform>,
    entities: Query<'w, 's, Entity, With<PartOfScene>>,
    client: ResMut<'w, RenetClient>,
    lobby: ResMut<'w, ClientPlayers>,
    network_mapping: ResMut<'w, NetworkMapping>,
    network_events: EventWriter<'w, NetworkEvent>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    spawn_player: EventWriter<'w, SpawnPlayer>,
    spawn_unit: EventWriter<'w, SpawnUnit>,
    spawn_projectile: EventWriter<'w, SpawnProjectile>,
    spawn_flag: EventWriter<'w, SpawnFlag>,
    game_state: ResMut<'w, NextState<GameState>>,
    player_joined: EventWriter<'w, PlayerJoinedLobby>,
    player_left: EventWriter<'w, PlayerLeftLobby>,
    player_checkbox: EventWriter<'w, PlayerCheckbox>,
}

fn client_sync_players(mut params: ClientSyncPlayersParams) {
    while let Some(message) = params.client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::SpawnPlayer(spawn) => {
                params.spawn_player.send(spawn);
            }
            ServerMessages::SpawnUnit(spawn) => {
                params.spawn_unit.send(spawn);
            }
            ServerMessages::SpawnProjectile(spawn) => {
                params.spawn_projectile.send(spawn);
            }
            ServerMessages::SpawnFlag(spawn) => {
                params.spawn_flag.send(spawn);
            }
            ServerMessages::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                if let Some(PlayerEntityMapping {
                    server_entity,
                    client_entity,
                }) = params.lobby.players.remove(&id)
                {
                    params.commands.entity(client_entity).despawn();
                    params.network_mapping.0.remove(&server_entity);
                }
            }
            ServerMessages::MeleeAttack {
                entity: server_entity,
            } => {
                if let Some(client_entity) = params.network_mapping.0.get(&server_entity) {
                    params.network_events.send(NetworkEvent {
                        entity: *client_entity,
                        change: Change::Attack,
                    });
                }
            }
            ServerMessages::DespawnEntity {
                entity: server_entity,
            } => {
                if let Some(client_entity) = params.network_mapping.0.remove(&server_entity) {
                    if let Some(mut entity) = params.commands.get_entity(client_entity) {
                        entity.despawn();
                    }
                }
            }
            ServerMessages::LoadGameScene {
                game_scene_type: map_type,
                players,
                units,
                projectiles,
            } => {
                println!("Loading map {:?}...", map_type);

                for entity in params.entities.iter() {
                    params.commands.entity(entity).despawn();
                }

                match map_type {
                    GameSceneType::Base(color) => {
                        let base = BaseScene::new();
                        params.commands.spawn((
                            base.main_building,
                            (
                                Mesh2dHandle(
                                    params
                                        .meshes
                                        .add(Rectangle::from_size(base.main_building.collider.0)),
                                ),
                                params.materials.add(color),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ));
                        params.commands.spawn((
                            base.archer_building,
                            (
                                Mesh2dHandle(
                                    params
                                        .meshes
                                        .add(Rectangle::from_size(base.archer_building.collider.0)),
                                ),
                                params.materials.add(color),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ));
                        params.commands.spawn((
                            base.warrior_building,
                            (
                                Mesh2dHandle(
                                    params.meshes.add(Rectangle::from_size(
                                        base.warrior_building.collider.0,
                                    )),
                                ),
                                params.materials.add(color),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ));
                        params.commands.spawn((
                            base.pikeman_building,
                            (
                                Mesh2dHandle(
                                    params.meshes.add(Rectangle::from_size(
                                        base.pikeman_building.collider.0,
                                    )),
                                ),
                                params.materials.add(color),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ));
                        params.commands.spawn((
                            base.left_wall,
                            (
                                Mesh2dHandle(
                                    params
                                        .meshes
                                        .add(Rectangle::from_size(base.left_wall.collider.0)),
                                ),
                                params.materials.add(color),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ));
                        params.commands.spawn((
                            base.right_wall,
                            (
                                Mesh2dHandle(
                                    params
                                        .meshes
                                        .add(Rectangle::from_size(base.right_wall.collider.0)),
                                ),
                                params.materials.add(color),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ));

                        params.commands.spawn((
                            base.left_spawn_point,
                            (
                                Mesh2dHandle(
                                    params.meshes.add(Rectangle::from_size(
                                        base.left_spawn_point.collider.0,
                                    )),
                                ),
                                params.materials.add(Color::from(YELLOW)),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ));
                        params.commands.spawn((
                            base.right_spawn_point,
                            (
                                Mesh2dHandle(
                                    params.meshes.add(Rectangle::from_size(
                                        base.left_spawn_point.collider.0,
                                    )),
                                ),
                                params.materials.add(Color::from(YELLOW)),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ));
                    }
                    GameSceneType::Camp => todo!(),
                };
                players.into_iter().for_each(|spawn| {
                    params.spawn_player.send(spawn);
                });
                units.into_iter().for_each(|spawn| {
                    params.spawn_unit.send(spawn);
                });
                projectiles.into_iter().for_each(|spawn| {
                    params.spawn_projectile.send(spawn);
                });

                params.game_state.set(GameState::GameSession);
            }
            ServerMessages::PlayerJoinedLobby {
                id,
                ready_state: checkbox_state,
            } => {
                params.player_joined.send(PlayerJoinedLobby {
                    id,
                    ready_state: checkbox_state,
                });
            }
            ServerMessages::LobbyPlayerReadyState {
                id,
                ready_state: checkbox_state,
            } => {
                params
                    .player_checkbox
                    .send(PlayerCheckbox { id, checkbox_state });
            }
            ServerMessages::PlayerLeftLobby { id } => {
                params.player_left.send(PlayerLeftLobby(id));
            }
        }
    }

    while let Some(message) = params
        .client
        .receive_message(ServerChannel::NetworkedEntities)
    {
        let maybe_net_entities: Result<NetworkedEntities, _> = bincode::deserialize(&message);
        match maybe_net_entities {
            Ok(networked_entities) => {
                for i in 0..networked_entities.entities.len() {
                    if let Some(client_entity) = params
                        .network_mapping
                        .0
                        .get(&networked_entities.entities[i].entity)
                    {
                        let network_entity = &networked_entities.entities[i];

                        if let Ok(mut transform) = params.transforms.get_mut(*client_entity) {
                            transform.translation = network_entity.translation.into();
                        }

                        params.network_events.send(NetworkEvent {
                            entity: *client_entity,
                            change: Change::Rotation(network_entity.rotation.clone()),
                        });

                        params.network_events.send(NetworkEvent {
                            entity: *client_entity,
                            change: Change::Movement(network_entity.moving),
                        });
                    }
                }
            }
            Err(error) => error!("Error on deserialize: {}", error),
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
