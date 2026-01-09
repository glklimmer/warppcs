use bevy::prelude::*;

use bevy::{ecs::entity::MapEntities, platform::collections::HashMap};
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientEventAppExt, ClientId, ClientMessageAppExt, FromClient, Replicated,
    SendMode, ServerEventAppExt, ServerTriggerExt, ToClients,
};
use serde::{Deserialize, Serialize};
use shared::enum_map::*;
use shared::{GameSceneId, GameState};

use crate::create_server::CreateServerPlugin;

pub mod create_server;

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CreateServerPlugin)
            .init_resource::<ClientPlayerMap>()
            .replicate::<PlayerColor>()
            .add_client_message::<LobbyMessage>(Channel::Ordered)
            .add_client_event::<ClientReady>(Channel::Ordered)
            .add_server_event::<GameStarted>(Channel::Ordered)
            .add_mapped_server_event::<SetLocalPlayer>(Channel::Ordered)
            .add_observer(on_client_ready)
            .add_observer(game_started);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct PendingPlayers(HashMap<Entity, u64>);

/// Key is NetworkEntity
/// Value is PlayerEntity
#[derive(Resource, DerefMut, Deref, Default, Reflect)]
pub struct ClientPlayerMap(HashMap<ClientId, Entity>);

impl ClientPlayerMap {
    pub fn get_network_entity(&self, value: &Entity) -> Result<&ClientId> {
        self.iter()
            .find_map(|(key, val)| if val == value { Some(key) } else { None })
            .ok_or("Network entity not found for player entity".into())
    }
}

pub trait ClientPlayerMapExt {
    fn get_player(&self, entity: &ClientId) -> Result<&Entity>;
}

impl ClientPlayerMapExt for ClientPlayerMap {
    fn get_player(&self, entity: &ClientId) -> Result<&Entity> {
        self.get(entity).ok_or("Player not found".into())
    }
}

#[derive(Event, Debug, Serialize, Deserialize)]
pub struct ClientReady(pub usize);

#[derive(Event, Default, Debug, Deserialize, Serialize)]
pub struct GameStarted(pub usize);

#[derive(Event, Clone, Copy, Debug, Deserialize, Serialize, Deref, DerefMut)]
pub struct SetLocalPlayer(pub Entity);

impl MapEntities for SetLocalPlayer {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.get_mapped(self.0);
    }
}

#[derive(Component)]
pub struct Disconnected;

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Mappable, Serialize, Deserialize,
)]
#[require(Replicated)]
pub enum PlayerColor {
    #[default]
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    Orange,
    Cyan,
    Magenta,
    Pink,
    Brown,
    Teal,
    Gray,
}

#[derive(Debug, Deserialize, Message, Serialize)]
pub enum LobbyMessage {
    StartGame,
}

fn game_started(_started: On<GameStarted>, mut next_game_state: ResMut<NextState<GameState>>) {
    next_game_state.set(GameState::GameSession);
}

fn on_client_ready(
    ready: On<FromClient<ClientReady>>,
    mut commands: Commands,
    client_player_map: Res<ClientPlayerMap>,
    players_query: Query<&GameSceneId>,
) {
    let client_id = &ready.client_id;
    if let Some(player_entity) = client_player_map.get(client_id) {
        if let Ok(game_scene_id) = players_query.get(*player_entity) {
            info!(
                "Client for reconnected player {:?} is ready. Re-inserting GameSceneId.",
                player_entity
            );
            commands.entity(*player_entity).insert(*game_scene_id);
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(*client_id),
                message: GameStarted(0),
            });
        } else {
            info!(
                "Client for new player {:?} is ready. Inserting lobby GameSceneId.",
                player_entity
            );
            commands.entity(*player_entity).insert(GameSceneId::lobby());
        }
    }
}
