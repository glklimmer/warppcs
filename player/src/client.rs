use bevy::prelude::*;

use animations::king::KingAnimation;
use bevy::platform::collections::HashMap;
use bevy_parallax::{CameraFollow, LinearAxisStrategy, TranslationStrategy};
use bevy_replicon::{
    prelude::{
        ClientId, ClientTriggerExt, ClientVisibility, SendMode, ServerTriggerExt, ToClients,
    },
    server::AuthorizedClient,
};
use lobby::{
    ClientPlayerMap, ClientReady, ControlledPlayer, Disconnected, PendingPlayers, PlayerColor,
    SetLocalPlayer,
};
use shared::{GameSceneId, GameState, Owner, enum_map::EnumIter, map::Layers};

use crate::Player;

pub(crate) struct Client;

impl Plugin for Client {
    fn build(&self, app: &mut App) {
        app.add_observer(on_created)
            .add_observer(spawn_clients)
            .add_observer(update_visibility)
            .add_observer(hide_on_remove)
            .add_observer(init_local_player);
    }
}

fn on_created(
    _: On<Add, Server>,
    mut client_player_map: ResMut<ClientPlayerMap>,
    mut commands: Commands,
) {
    info!("Successfully created server");

    let server_player = commands
        .spawn(Player {
            id: 0,
            color: *fastrand::choice(PlayerColor::all_variants())
                .expect("No PlayerColor available"),
        })
        .id();

    client_player_map.insert(ClientId::Server, server_player);

    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        message: SetLocalPlayer(server_player),
    });
}

fn spawn_clients(
    trigger: On<Add, AuthorizedClient>,
    mut visibility: Query<&mut ClientVisibility>,
    mut client_player_map: ResMut<ClientPlayerMap>,
    mut commands: Commands,
    mut pending_players: ResMut<PendingPlayers>,
    disconnected_players: Query<(Entity, &Player), With<Disconnected>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let client_id = ClientId::Client(trigger.entity);
    let new_player_id = if let Some(id) = pending_players.remove(&trigger.entity) {
        id
    } else {
        trigger.entity.to_bits()
    };

    // Try to find a disconnected player with the same ID
    if let Some((player_entity, _)) = disconnected_players
        .iter()
        .find(|(_, player)| player.id == new_player_id)
    {
        // Player found, reconnect them
        commands.entity(player_entity).remove::<Disconnected>();
        client_player_map.insert(client_id, player_entity);

        for mut client_visibility in visibility.iter_mut() {
            client_visibility.set_visibility(player_entity, true);
        }

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(client_id),
            message: SetLocalPlayer(player_entity),
        });

        info!(
            "Player {:?} reconnected with id {}.",
            player_entity, new_player_id
        );

        // TODO: wait until client has set up ControlledPlayer

        // After this player is reconnected, there is one less disconnected player.
        // If there are no more disconnected players, unpause the game.
        if disconnected_players.iter().count() == 1 {
            game_state.set(GameState::GameSession);
            info!("All players reconnected, resuming game.");
        }
        return;
    }

    // No disconnected player found, spawn a new one
    let color = fastrand::choice(PlayerColor::all_variants()).unwrap();
    let player = commands.spawn_empty().id();
    commands.entity(player).insert((
        Player {
            id: new_player_id,
            color: *color,
        },
        Transform::from_xyz(250.0, 0.0, Layers::Player.as_f32()),
        Owner::Player(player),
    ));

    client_player_map.insert(client_id, player);

    for mut client_visibility in visibility.iter_mut() {
        client_visibility.set_visibility(player, true);
    }

    info!("New player {:?} spawned with id {}.", player, new_player_id);
    commands.server_trigger(ToClients {
        mode: SendMode::Direct(client_id),
        message: SetLocalPlayer(player),
    });
    // TODO: not sure wether replicated on MapDiscovery or sending all events here
}

fn update_visibility(
    trigger: On<Insert, GameSceneId>,
    client_player_map: Res<ClientPlayerMap>,
    mut visibility_query: Query<&mut ClientVisibility>,
    players_query: Query<(Entity, &GameSceneId), With<Player>>,
    others: Query<(Entity, &GameSceneId)>,
    player_check: Query<(), With<Player>>,
) -> Result {
    let entity = trigger.entity;
    let (_, new_entity_scene_id) = others.get(entity)?;

    if player_check.get(entity).is_ok() {
        let player_scenes: HashMap<Entity, GameSceneId> = players_query
            .iter()
            .map(|(entity, game_scene_id)| (entity, *game_scene_id))
            .collect();

        for (player_entity, _player_scene_id) in players_query.iter() {
            let client_entity = match client_player_map.get_network_entity(&player_entity)? {
                ClientId::Client(entity) => *entity,
                _ => continue,
            };

            if let Ok(mut visibility) = visibility_query.get_mut(client_entity) {
                if player_entity.eq(&entity) {
                    let player_scene_id = player_scenes
                        .get(&entity)
                        .ok_or("GameSceneId for player not found")?;
                    for (other_entity, other_scene_id) in &others {
                        visibility.set_visibility(other_entity, other_scene_id.eq(player_scene_id));
                    }
                } else {
                    let player_scene_id = player_scenes
                        .get(&player_entity)
                        .ok_or("GameSceneId for player not found")?;
                    visibility.set_visibility(entity, player_scene_id.eq(new_entity_scene_id));
                }
            }
        }
    } else {
        for (player_entity, player_scene_id) in players_query.iter() {
            let client_entity = match client_player_map.get_network_entity(&player_entity)? {
                ClientId::Client(entity) => *entity,
                _ => continue,
            };
            if let Ok(mut visibility) = visibility_query.get_mut(client_entity) {
                visibility.set_visibility(entity, player_scene_id.eq(new_entity_scene_id));
            }
        }
    }
    Ok(())
}

fn hide_on_remove(
    trigger: On<Remove, GameSceneId>,
    mut visibility_query: Query<&mut ClientVisibility>,
    players_query: Query<Entity, With<Player>>,
    client_player_map: Res<ClientPlayerMap>,
) {
    let entity = trigger.entity;
    for player_entity in players_query.iter() {
        if let Ok(client_id) = client_player_map.get_network_entity(&player_entity) {
            let client_entity = match client_id {
                ClientId::Client(e) => e,
                _ => continue,
            };
            if let Ok(mut visibility) = visibility_query.get_mut(*client_entity) {
                visibility.set_visibility(entity, player_entity == entity);
            }
        }
    }
}

fn init_local_player(
    trigger: On<SetLocalPlayer>,
    camera: Query<Entity, With<Camera>>,
    mut commands: Commands,
) -> Result {
    let player = trigger.entity();
    let mut player_commands = commands.entity(player);
    player_commands.insert((
        ControlledPlayer,
        KingAnimation::default(),
        SpatialListener::new(50.0),
    ));
    commands.entity(camera.single()?).insert(
        CameraFollow::fixed(player)
            .with_offset(Vec2 { x: 50., y: 50. })
            .with_translation(TranslationStrategy {
                x: LinearAxisStrategy::P(0.03),
                y: LinearAxisStrategy::P(0.9),
            }),
    );
    commands.client_trigger(ClientReady(0));
    Ok(())
}
