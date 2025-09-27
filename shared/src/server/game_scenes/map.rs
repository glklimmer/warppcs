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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExitType {
    PlayerLeft,
    PlayerRight,
    TraversalLeft,
    TraversalRight,
    TJunctionLeft,
    TJunctionMiddle,
    TJunctionRight,
}

#[derive(Resource, Clone, Serialize, Deserialize, Default, Deref, DerefMut)]
pub struct MapGraph(pub Graph<GameScene, (ExitType, ExitType), Undirected>);

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

        let mut graph = Graph::<GameScene, (ExitType, ExitType), Undirected>::new_undirected();

        // 1. Create all nodes and store their indices
        let mut player_nodes = Vec::with_capacity(num_players);
        let mut tj_a_nodes = Vec::with_capacity(num_players);
        let mut tj_b_nodes = Vec::with_capacity(num_players);
        let mut t1_nodes = Vec::with_capacity(num_players); // Outer traversals
        let mut t2_nodes = Vec::with_capacity(num_players); // Inner traversals

        for i in 0..num_players {
            // Player
            let frac = i as f32 / num_players as f32;
            let angle = frac * std::f32::consts::TAU;
            let pos = Vec2::new(radius * angle.cos(), radius * angle.sin());
            let p_idx = graph.add_node(GameScene {
                scene: SceneType::Player {
                    player: players[i],
                    left: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: pos,
            });
            player_nodes.push(p_idx);

            // Intermediate nodes for segment i
            let next_frac = (i as f32 + 1.0) / num_players as f32;
            let next_angle = next_frac * std::f32::consts::TAU;
            let next_pos = Vec2::new(radius * next_angle.cos(), radius * next_angle.sin());

            let tj_a_pos = pos.lerp(next_pos, 0.25);
            let tj_b_pos = pos.lerp(next_pos, 0.75);

            let mid_point = pos.lerp(next_pos, 0.5);
            let segment_vec = next_pos - pos;
            let offset_dir = segment_vec.perp().normalize_or_zero();
            let offset_dist = segment_vec.length() * 0.2;

            let t1_pos = mid_point + offset_dir * offset_dist; // Outer
            let t2_pos = mid_point - offset_dir * offset_dist; // Inner

            let tj_a_idx = graph.add_node(GameScene {
                scene: SceneType::TJunction {
                    left: commands.spawn_empty().id(),
                    middle: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: tj_a_pos,
            });
            tj_a_nodes.push(tj_a_idx);
            let tj_b_idx = graph.add_node(GameScene {
                scene: SceneType::TJunction {
                    left: commands.spawn_empty().id(),
                    middle: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: tj_b_pos,
            });
            tj_b_nodes.push(tj_b_idx);
            let t1_idx = graph.add_node(GameScene {
                scene: SceneType::Traversal {
                    left: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: t1_pos,
            });
            t1_nodes.push(t1_idx);
            let t2_idx = graph.add_node(GameScene {
                scene: SceneType::Traversal {
                    left: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: t2_pos,
            });
            t2_nodes.push(t2_idx);
        }

        // 2. Add edges with deterministic exit types
        for i in 0..num_players {
            let p_idx = player_nodes[i];
            let next_p_idx = player_nodes[(i + 1) % num_players];
            let tj_a_idx = tj_a_nodes[i];
            let tj_b_idx = tj_b_nodes[i];
            let t1_idx = t1_nodes[i];
            let t2_idx = t2_nodes[i];

            // Player -> TJunction connections
            graph.add_edge(
                p_idx,
                tj_a_idx,
                (ExitType::PlayerRight, ExitType::TJunctionMiddle),
            );
            graph.add_edge(
                next_p_idx,
                tj_b_idx,
                (ExitType::PlayerLeft, ExitType::TJunctionMiddle),
            );

            // TJunction -> Traversal connections
            graph.add_edge(
                tj_a_idx,
                t1_idx,
                (ExitType::TJunctionRight, ExitType::TraversalLeft),
            );
            graph.add_edge(
                tj_a_idx,
                t2_idx,
                (ExitType::TJunctionLeft, ExitType::TraversalRight),
            );
            graph.add_edge(
                tj_b_idx,
                t1_idx,
                (ExitType::TJunctionLeft, ExitType::TraversalRight),
            );
            graph.add_edge(
                tj_b_idx,
                t2_idx,
                (ExitType::TJunctionRight, ExitType::TraversalLeft),
            );
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
    let map = MapGraph::circular(
        commands.reborrow(),
        players,
        100. + 25. * num_players as f32,
    );

    commands.insert_resource(map.clone());
    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        event: LoadMap(map),
    });
}

