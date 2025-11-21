use bevy::prelude::*;

use bevy_replicon::{
    prelude::{FromClient, SendMode, ServerTriggerExt, ToClients, server_or_singleplayer},
    server::ServerSet,
};
use petgraph::{Graph, Undirected};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{ClientPlayerMap, GameState, Player, networking::LobbyEvent};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            init_world
                .after(ServerSet::Receive)
                .run_if(server_or_singleplayer)
                .run_if(in_state(GameState::MainMenu)),
        );
    }
}

#[derive(Event, Deref)]
pub struct InitWorld(WorldGraph);

#[derive(Event, Deref, Serialize, Deserialize)]
pub struct InitPlayerMapNode(GameScene);

#[derive(Event, Deref, Serialize, Deserialize)]
pub struct RevealMapNode(GameScene);

impl RevealMapNode {
    pub fn to(game_scene: GameScene) -> Self {
        Self(game_scene)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitType {
    Left,
    Right,
}

#[derive(Clone, Default, Deref, DerefMut)]
pub struct WorldGraph(Graph<GameScene, (ExitType, ExitType), Undirected>);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SceneType {
    Player { player: Entity, exit: Entity },
    Camp { left: Entity, right: Entity },
    Meadow { left: Entity, right: Entity },
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Copy)]
pub struct GameScene {
    pub scene: SceneType,
    pub position: Vec2,
}

#[derive(Default, Clone, Deref)]
struct PlayerGameScenes(HashMap<Entity, GameScene>);

impl WorldGraph {
    fn circular(
        mut commands: Commands,
        players: Vec<Entity>,
        radius: f32,
    ) -> (WorldGraph, PlayerGameScenes) {
        let num_players = players.len();
        if num_players == 0 {
            return (WorldGraph::default(), PlayerGameScenes::default());
        }

        let mut graph = Graph::<GameScene, (ExitType, ExitType), Undirected>::new_undirected();
        let mut player_game_scenes = HashMap::new();

        // Create all nodes and store their indices
        let mut player_nodes = Vec::with_capacity(num_players);
        let mut tj_a_nodes = Vec::with_capacity(num_players);
        let mut tj_b_nodes = Vec::with_capacity(num_players);
        let mut inner_nodes = Vec::with_capacity(num_players);
        let mut outer_nodes = Vec::with_capacity(num_players);

        for (i, ..) in players.iter().enumerate() {
            // Player
            let frac = i as f32 / num_players as f32;
            let angle = frac * std::f32::consts::TAU;
            let pos = Vec2::new(radius * angle.cos(), radius * angle.sin());
            let game_scene = GameScene {
                scene: SceneType::Player {
                    player: players[i],
                    exit: commands.spawn_empty().id(),
                },
                position: pos,
            };
            let p_idx = graph.add_node(game_scene);
            player_nodes.push(p_idx);
            player_game_scenes.insert(players[i], game_scene);

            // Intermediate nodes for segment i
            let next_frac = (i as f32 + 1.0) / num_players as f32;
            let next_angle = next_frac * std::f32::consts::TAU;
            let next_pos = Vec2::new(radius * next_angle.cos(), radius * next_angle.sin());

            // Outward push logic
            let segment_midpoint = pos.lerp(next_pos, 0.5);
            let outward_dir = segment_midpoint.normalize_or_zero();
            let push_out_dist = if num_players == 2 {
                -80.
            } else if num_players == 3 {
                20.
            } else {
                0.
            };

            let tj_a_pos = pos.lerp(next_pos, 0.25) + outward_dir * push_out_dist;
            let tj_b_pos = pos.lerp(next_pos, 0.75) + outward_dir * push_out_dist;

            let mid_point = pos.lerp(next_pos, 0.5);
            let segment_vec = next_pos - pos;
            let offset_dir = segment_vec.perp().normalize_or_zero(); // Points inward
            let offset_dist = segment_vec.length() * 0.2;

            let inner_pos = mid_point + offset_dir * offset_dist + outward_dir * push_out_dist;
            let outer_pos = mid_point - offset_dir * offset_dist + outward_dir * push_out_dist;

            let tj_a_idx = graph.add_node(GameScene {
                scene: SceneType::Camp {
                    left: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: tj_a_pos,
            });
            tj_a_nodes.push(tj_a_idx);
            let tj_b_idx = graph.add_node(GameScene {
                scene: SceneType::Camp {
                    left: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: tj_b_pos,
            });
            tj_b_nodes.push(tj_b_idx);

            let outer_idx = graph.add_node(GameScene {
                scene: SceneType::Meadow {
                    left: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: outer_pos,
            });
            outer_nodes.push(outer_idx);

            let inner_scene = SceneType::Camp {
                left: commands.spawn_empty().id(),
                right: commands.spawn_empty().id(),
            };
            let inner_idx = graph.add_node(GameScene {
                scene: inner_scene,
                position: inner_pos,
            });
            inner_nodes.push(inner_idx);
        }

        // Add edges with deterministic exit types
        for i in 0..num_players {
            let p_idx = player_nodes[i];
            let next_p_idx = player_nodes[(i + 1) % num_players];
            let tj_a_idx = tj_a_nodes[i];
            let tj_b_idx = tj_b_nodes[i];
            let t1_idx = inner_nodes[i];
            let t2_idx = outer_nodes[i];

            // Player connections
            graph.add_edge(p_idx, tj_a_idx, (ExitType::Right, ExitType::Left));
            graph.add_edge(next_p_idx, tj_b_idx, (ExitType::Right, ExitType::Left));

            // Outer Traversal (outer_idx) connections
            graph.add_edge(tj_a_idx, t2_idx, (ExitType::Left, ExitType::Right));
            graph.add_edge(tj_b_idx, t2_idx, (ExitType::Right, ExitType::Left));

            // Inner Node (inner_idx) connections
            graph.add_edge(tj_a_idx, t1_idx, (ExitType::Right, ExitType::Left));
            graph.add_edge(tj_b_idx, t1_idx, (ExitType::Left, ExitType::Right));
        }

        // Add inner circle connections
        for i in 0..num_players {
            let node_a = inner_nodes[i];
            let node_b = inner_nodes[(i + 1) % num_players];
            if num_players > 2 {
                graph.add_edge(node_a, node_b, (ExitType::Right, ExitType::Left));
            } else {
                graph.add_edge(node_a, node_b, (ExitType::Left, ExitType::Right));
                // graph.add_edge(node_a, node_b, (ExitType::Right, ExitType::Left));
            }
        }

        (WorldGraph(graph), PlayerGameScenes(player_game_scenes))
    }
}

fn init_world(
    mut lobby_events: EventReader<FromClient<LobbyEvent>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    players: Query<Entity, With<Player>>,
    client_player_map: Res<ClientPlayerMap>,
    mut commands: Commands,
) -> Result {
    let Some(FromClient { event, .. }) = lobby_events.read().next() else {
        return Ok(());
    };

    #[allow(irrefutable_let_patterns)]
    let LobbyEvent::StartGame = event else {
        return Ok(());
    };

    next_game_state.set(GameState::GameSession);

    let players: Vec<Entity> = players.iter().collect();
    let num_players = players.len();
    let (map, player_game_scenes) = WorldGraph::circular(
        commands.reborrow(),
        players,
        100. + 25. * num_players as f32,
    );

    commands.trigger(InitWorld(map));

    for (client, player) in client_player_map.iter() {
        let game_scene = player_game_scenes
            .get(player)
            .ok_or("GameScene for player not found")?;

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*client),
            event: InitPlayerMapNode(*game_scene),
        });
    }
    Ok(())
}
