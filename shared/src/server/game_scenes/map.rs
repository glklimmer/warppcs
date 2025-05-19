use bevy::{ecs::entity::MapEntities, prelude::*};

use bevy_replicon::{
    prelude::{FromClient, SendMode, ServerTriggerExt, ToClients, server_or_singleplayer},
    server::ServerSet,
};
use petgraph::{Graph, Undirected};
use serde::{Deserialize, Serialize};

use crate::{Player, networking::LobbyEvent};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapGraph>().add_systems(
            PreUpdate,
            init_map
                .after(ServerSet::Receive)
                .run_if(server_or_singleplayer),
        );
    }
}

#[derive(Event, Serialize, Deserialize, Deref)]
pub struct LoadMap(pub MapGraph);

#[derive(Resource, Clone, Serialize, Deserialize, Default, Deref, DerefMut)]
pub struct MapGraph(pub Graph<GameScene, (), Undirected>);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SceneType {
    Player {
        player: Entity,
        left: Entity,
        right: Entity,
    },
    Bandit {
        left: Entity,
        right: Entity,
    },
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct GameScene {
    pub scene: SceneType,
    pub position: Vec2,
}

impl MapGraph {
    pub fn circular(mut commands: Commands, players: Vec<Entity>, radius: f32) -> MapGraph {
        let total = players.len() * 2;
        let mut graph = Graph::<GameScene, (), Undirected>::with_capacity(total, total);
        let mut indices = Vec::with_capacity(total);

        for node_index in 0..total {
            let frac = node_index as f32 / total as f32;
            let angle = frac * std::f32::consts::TAU;
            let pos = Vec2::new(radius * angle.cos(), radius * angle.sin());

            let node = if node_index % 2 == 0 {
                let player_idx = node_index / 2;
                let player_entity = players[player_idx];
                graph.add_node(GameScene {
                    scene: SceneType::Player {
                        player: player_entity,
                        left: commands.spawn_empty().id(),
                        right: commands.spawn_empty().id(),
                    },
                    position: pos,
                })
            } else {
                graph.add_node(GameScene {
                    scene: SceneType::Bandit {
                        left: commands.spawn_empty().id(),
                        right: commands.spawn_empty().id(),
                    },
                    position: pos,
                })
            };

            indices.push(node);
        }

        for i in 0..total {
            let a = indices[i];
            let b = indices[(i + 1) % total];
            graph.add_edge(a, b, ());
        }

        MapGraph(graph)
    }
}

fn init_map(
    mut lobby_events: EventReader<FromClient<LobbyEvent>>,
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
) {
    let Some(FromClient { event, .. }) = lobby_events.read().next() else {
        return;
    };

    #[allow(irrefutable_let_patterns)]
    let LobbyEvent::StartGame = event else {
        return;
    };

    let players: Vec<Entity> = players.iter().collect();
    let num_players = players.len();
    let map = MapGraph::circular(commands.reborrow(), players, 25. + 25. * num_players as f32);

    commands.insert_resource(map.clone());
    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        event: LoadMap(map),
    });
}
