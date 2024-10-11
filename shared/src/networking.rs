use bevy::prelude::*;

use bevy_renet::renet::{ChannelConfig, ClientId, ConnectionConfig, SendType};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::map::GameSceneType;

pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UnitType {
    Shieldwarrior,
    Pikeman,
    Archer,
}

#[derive(Component, Clone)]
pub struct Unit {
    pub health: f32,
    pub unit_type: UnitType,
    pub swing_timer: Timer,
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub enum PlayerCommand {
    StartGame,
    Interact,
    MeleeAttack,
    SpawnUnit(UnitType),
}

pub enum ClientChannel {
    Input,
    Command,
}
pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
}

#[derive(Debug, Component, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct Owner(pub ClientId);

#[derive(Debug, Component, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum ProjectileType {
    Arrow,
}

#[derive(Debug, Component, Serialize, Deserialize, Copy, Clone)]
pub enum PlayerSkin {
    Warrior,
    Monster,
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub struct SpawnPlayer {
    pub id: ClientId,
    pub entity: Entity,
    pub translation: [f32; 3],
    pub skin: PlayerSkin,
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub struct SpawnUnit {
    pub owner: Owner,
    pub entity: Entity,
    pub unit_type: UnitType,
    pub translation: [f32; 3],
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub struct SpawnProjectile {
    pub entity: Entity,
    pub projectile_type: ProjectileType,
    pub translation: [f32; 3],
    pub direction: [f32; 2],
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    SpawnPlayer(SpawnPlayer),
    SpawnUnit(SpawnUnit),
    SpawnProjectile(SpawnProjectile),
    PlayerRemove {
        id: ClientId,
    },
    DespawnEntity {
        entity: Entity,
    },
    LoadGameScene {
        game_scene_type: GameSceneType,
        players: Vec<SpawnPlayer>,
        units: Vec<SpawnUnit>,
        projectiles: Vec<SpawnProjectile>,
    },
    MeleeAttack {
        entity: Entity,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum Facing {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rotation {
    LeftRight { facing: Option<Facing> },
    Free { angle: f32 },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkEntity {
    pub entity: Entity,
    pub translation: [f32; 3],
    pub rotation: Rotation,
    pub moving: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedEntities {
    pub entities: Vec<NetworkEntity>,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Command => 0,
            ClientChannel::Input => 1,
        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::Input.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::ZERO,
                },
            },
            ChannelConfig {
                channel_id: Self::Command.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::ZERO,
                },
            },
        ]
    }
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::NetworkedEntities => 0,
            ServerChannel::ServerMessages => 1,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::NetworkedEntities.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Self::ServerMessages.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
        ]
    }
}

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: ClientChannel::channels_config(),
        server_channels_config: ServerChannel::channels_config(),
    }
}
