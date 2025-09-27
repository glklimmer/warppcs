use bevy::prelude::*;

use bevy_replicon::{
    prelude::{FromClient, SendMode, ServerTriggerExt, ToClients, server_or_singleplayer},
    server::ServerSet,
};
use petgraph::{Graph, Undirected};
use serde::{Deserialize, Serialize};

use crate::{GameState, Player, networking::LobbyEvent};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapGraph>().add_systems(
            PreUpdate,
            init_map
                .after(ServerSet::Receive)
                .run_if(server_or_singleplayer)
                .run_if(in_state(GameState::MainMenu)),
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
    Traversal {
        left: Entity,
        right: Entity,
    },
    TJunction {
        left: Entity,
        middle: Entity,
        right: Entity,
    },
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Copy)]
pub struct GameScene {
    pub scene: SceneType,
    pub position: Vec2,
}

impl MapGraph {
    pub fn circular(mut commands: Commands, players: Vec<Entity>, radius: f32) -> MapGraph {
        let num_players = players.len();
        if num_players == 0 {
            return MapGraph::default();
        }

        let mut graph = Graph::<GameScene, (), Undirected>::new_undirected();

        // 1. Create all player nodes and store their indices and positions
        let mut player_node_indices = Vec::with_capacity(num_players);
        let mut player_positions = Vec::with_capacity(num_players);
        for i in 0..num_players {
            let frac = i as f32 / num_players as f32;
            let angle = frac * std::f32::consts::TAU;
            let pos = Vec2::new(radius * angle.cos(), radius * angle.sin());
            player_positions.push(pos);

            let node_idx = graph.add_node(GameScene {
                scene: SceneType::Player {
                    player: players[i],
                    left: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: pos,
            });
            player_node_indices.push(node_idx);
        }

        // 2. For each segment between players, create the 4 intermediate nodes and connect them.
        for i in 0..num_players {
            let current_player_idx = player_node_indices[i];
            let next_player_idx = player_node_indices[(i + 1) % num_players];

            let current_player_pos = player_positions[i];
            let next_player_pos = player_positions[(i + 1) % num_players];

            // Create the 4 intermediate nodes for the segment
            // TJunction near current player
            let tj_a_pos = current_player_pos.lerp(next_player_pos, 0.2);
            let tj_a_idx = graph.add_node(GameScene {
                scene: SceneType::TJunction {
                    left: commands.spawn_empty().id(),
                    middle: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: tj_a_pos,
            });

            // TJunction near next player
            let tj_b_pos = current_player_pos.lerp(next_player_pos, 0.8);
            let tj_b_idx = graph.add_node(GameScene {
                scene: SceneType::TJunction {
                    left: commands.spawn_empty().id(),
                    middle: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: tj_b_pos,
            });

            // Traversal nodes
            let mid_point = current_player_pos.lerp(next_player_pos, 0.5);
            let segment_vec = next_player_pos - current_player_pos;
            let offset_dir = segment_vec.perp().normalize_or_zero();
            let offset_dist = segment_vec.length() * 0.3; // 30% of segment length

            let traversal1_pos = mid_point + offset_dir * offset_dist;
            let traversal1_idx = graph.add_node(GameScene {
                scene: SceneType::Traversal {
                    left: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: traversal1_pos,
            });

            let traversal2_pos = mid_point - offset_dir * offset_dist;
            let traversal2_idx = graph.add_node(GameScene {
                scene: SceneType::Traversal {
                    left: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: traversal2_pos,
            });

            // 3. Add edges
            // Connect players to their respective T-junctions
            graph.add_edge(current_player_idx, tj_a_idx, ());
            graph.add_edge(next_player_idx, tj_b_idx, ());

            // Connect T-junctions to both traversal nodes to form the small circle
            graph.add_edge(tj_a_idx, traversal1_idx, ());
            graph.add_edge(tj_a_idx, traversal2_idx, ());
            graph.add_edge(tj_b_idx, traversal1_idx, ());
            graph.add_edge(tj_b_idx, traversal2_idx, ());
        }

        MapGraph(graph)
    }
}

fn init_map(
    mut lobby_events: EventReader<FromClient<LobbyEvent>>,
    mut commands: Commands,
    mut next_game_state: ResMut<NextState<GameState>>,
    players: Query<Entity, With<Player>>,
) {
    let Some(FromClient { event, .. }) = lobby_events.read().next() else {
        return;
    };

    #[allow(irrefutable_let_patterns)]
    let LobbyEvent::StartGame = event else {
        return;
    };

    next_game_state.set(GameState::GameSession);

    let players: Vec<Entity> = players.iter().collect();
    let num_players = players.len();
    let map = MapGraph::circular(commands.reborrow(), players, 25. + 25. * num_players as f32);

    commands.insert_resource(map.clone());
    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        event: LoadMap(map),
    });
}
