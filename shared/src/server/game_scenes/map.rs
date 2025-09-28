use bevy::{ecs::entity::MapEntities, prelude::*};

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

impl MapEntities for LoadMap {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for node in self.0.node_weights_mut() {
            node.map_entities(entity_mapper);
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExitType {
    PlayerLeft,
    PlayerRight,
    TraversalLeft,
    TraversalRight,
    TJunctionLeft,
    TJunctionMiddle,
    TJunctionRight,
    DoubleConnectionLeft,
    DoubleConnectionLeftConn,
    DoubleConnectionRightConn,
    DoubleConnectionRight,
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
    DoubleConnection {
        left: Entity,
        left_connection: Entity,
        right_connection: Entity,
        right: Entity,
    },
}

impl MapEntities for SceneType {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        match self {
            SceneType::Player {
                player,
                left,
                right,
            } => {
                *player = entity_mapper.get_mapped(*player);
                *left = entity_mapper.get_mapped(*left);
                *right = entity_mapper.get_mapped(*right);
            }
            SceneType::Traversal { left, right } => {
                *left = entity_mapper.get_mapped(*left);
                *right = entity_mapper.get_mapped(*right);
            }
            SceneType::TJunction {
                left,
                middle,
                right,
            } => {
                *left = entity_mapper.get_mapped(*left);
                *middle = entity_mapper.get_mapped(*middle);
                *right = entity_mapper.get_mapped(*right);
            }
            SceneType::DoubleConnection {
                left,
                left_connection,
                right_connection,
                right,
            } => {
                *left = entity_mapper.get_mapped(*left);
                *left_connection = entity_mapper.get_mapped(*left_connection);
                *right_connection = entity_mapper.get_mapped(*right_connection);
                *right = entity_mapper.get_mapped(*right);
            }
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Copy)]
pub struct GameScene {
    pub scene: SceneType,
    pub position: Vec2,
}

impl MapEntities for GameScene {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.scene.map_entities(entity_mapper);
    }
}

impl MapGraph {
    pub fn circular(mut commands: Commands, players: Vec<Entity>, radius: f32) -> MapGraph {
        let num_players = players.len();
        if num_players == 0 {
            return MapGraph::default();
        }

        let mut graph = Graph::<GameScene, (ExitType, ExitType), Undirected>::new_undirected();

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

            let outer_idx = graph.add_node(GameScene {
                scene: SceneType::Traversal {
                    left: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                },
                position: outer_pos,
            });
            outer_nodes.push(outer_idx);

            let inner_scene = if num_players > 2 {
                SceneType::DoubleConnection {
                    left: commands.spawn_empty().id(),
                    left_connection: commands.spawn_empty().id(),
                    right_connection: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                }
            } else {
                SceneType::TJunction {
                    left: commands.spawn_empty().id(),
                    middle: commands.spawn_empty().id(),
                    right: commands.spawn_empty().id(),
                }
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

            // TJunction -> Outer Traversal (outer_idx) connections
            graph.add_edge(
                tj_a_idx,
                t2_idx,
                (ExitType::TJunctionLeft, ExitType::TraversalRight),
            );
            graph.add_edge(
                tj_b_idx,
                t2_idx,
                (ExitType::TJunctionRight, ExitType::TraversalLeft),
            );

            // TJunction -> Inner Node (inner_idx) connections
            if num_players > 2 {
                graph.add_edge(
                    tj_a_idx,
                    t1_idx,
                    (ExitType::TJunctionRight, ExitType::DoubleConnectionLeft),
                );
                graph.add_edge(
                    tj_b_idx,
                    t1_idx,
                    (ExitType::TJunctionLeft, ExitType::DoubleConnectionRight),
                );
            } else {
                graph.add_edge(
                    tj_a_idx,
                    t1_idx,
                    (ExitType::TJunctionRight, ExitType::TJunctionLeft),
                );
                graph.add_edge(
                    tj_b_idx,
                    t1_idx,
                    (ExitType::TJunctionLeft, ExitType::TJunctionRight),
                );
            }
        }

        // Add inner circle connections
        for i in 0..num_players {
            let node_a = inner_nodes[i];
            let node_b = inner_nodes[(i + 1) % num_players];
            if num_players > 2 {
                graph.add_edge(
                    node_a,
                    node_b,
                    (
                        ExitType::DoubleConnectionRightConn,
                        ExitType::DoubleConnectionLeftConn,
                    ),
                );
            } else {
                graph.add_edge(
                    node_a,
                    node_b,
                    (ExitType::TJunctionMiddle, ExitType::TJunctionMiddle),
                );
            }
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
