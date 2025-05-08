use bevy::prelude::*;

use bevy_replicon::{
    prelude::{FromClient, SendMode, ServerTriggerExt, ToClients, server_or_singleplayer},
    server::ServerSet,
};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

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
pub struct LoadMap(MapGraph);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NodeType {
    Player,
    Bandit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub node_type: NodeType,
    pub position: Vec2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub a: usize,
    pub b: usize,
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize, Default)]
pub struct MapGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl MapGraph {
    pub fn circular(num_players: usize, num_bandits: usize, radius: f32) -> Self {
        let total = num_players + num_bandits;
        let mut nodes = Vec::with_capacity(total);
        let mut edges = Vec::with_capacity(total);

        for i in 0..total {
            let angle = (i as f32 / total as f32) * TAU;
            let pos = Vec2::new(radius * angle.cos(), radius * angle.sin());
            let node_type = if i % 2 == 0 {
                NodeType::Player
            } else {
                NodeType::Bandit
            };
            nodes.push(Node {
                id: i,
                node_type,
                position: pos,
            });
        }
        // connect in a ring
        for i in 0..total {
            edges.push(Edge {
                a: i,
                b: (i + 1) % total,
            });
        }

        MapGraph { nodes, edges }
    }

    pub fn star(
        num_players: usize,
        num_bandits: usize,
        outer_radius: f32,
        inner_radius: f32,
    ) -> Self {
        let mut nodes = Vec::with_capacity(num_players + num_bandits);
        let mut edges = Vec::new();

        for i in 0..num_players {
            let angle = (i as f32 / num_players as f32) * TAU;
            let pos = Vec2::new(outer_radius * angle.cos(), outer_radius * angle.sin());
            nodes.push(Node {
                id: i,
                node_type: NodeType::Player,
                position: pos,
            });
        }

        for j in 0..num_bandits {
            let idx = num_players + j;
            let angle = (j as f32 / num_bandits as f32) * TAU;
            let pos = Vec2::new(inner_radius * angle.cos(), inner_radius * angle.sin());
            nodes.push(Node {
                id: idx,
                node_type: NodeType::Bandit,
                position: pos,
            });
        }

        for p in 0..num_players {
            for b in num_players..(num_players + num_bandits) {
                edges.push(Edge { a: p, b });
            }
        }

        for b1 in num_players..(num_players + num_bandits) {
            for b2 in (b1 + 1)..(num_players + num_bandits) {
                edges.push(Edge { a: b1, b: b2 });
            }
        }

        MapGraph { nodes, edges }
    }
}

fn init_map(
    mut lobby_events: EventReader<FromClient<LobbyEvent>>,
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
) {
    let Some(FromClient {
        client_entity: _,
        event,
    }) = lobby_events.read().next()
    else {
        return;
    };

    #[allow(irrefutable_let_patterns)]
    let LobbyEvent::StartGame = &event else {
        return;
    };

    let num_players = players.iter().count();
    let map = MapGraph::circular(num_players, num_players, 25. + 25. * num_players as f32);

    commands.insert_resource(map.clone());
    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        event: LoadMap(map),
    });
}
