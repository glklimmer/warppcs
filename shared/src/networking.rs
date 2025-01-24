use bevy::prelude::*;

use super::enum_map::*;
use bevy_renet::renet::{ChannelConfig, ClientId, ConnectionConfig, SendType};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{
    map::{
        buildings::{BuildStatus, Building},
        scenes::SceneBuildingIndicator,
        GameSceneType,
    },
    projectile_collider, BoxCollider,
};

pub const PROTOCOL_ID: u64 = 7;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MultiplayerRoles {
    Host,
    Client,
    NotInGame,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone, Copy, Mappable)]
pub enum UnitType {
    Shieldwarrior,
    Pikeman,
    Archer,
    Bandit,
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub enum PlayerCommand {
    StartGame,
    Interact,
    MeleeAttack,
    FlagInteract,
    LobbyReadyState(Checkbox),
}

pub enum ClientChannel {
    Input,
    Command,
}
pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Faction {
    Player { client_id: ClientId },
    Bandits,
}

#[derive(Debug, Component, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct Owner {
    pub faction: Faction,
}

#[derive(Debug, Component, PartialEq, Serialize, Deserialize, Copy, Clone)]
#[require(BoxCollider(projectile_collider))]
pub enum ProjectileType {
    Arrow,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Inventory {
    pub gold: u16,
}

impl Default for Inventory {
    fn default() -> Self {
        Self { gold: 1000 }
    }
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct SpawnPlayer {
    pub id: ClientId,
    pub entity: Entity,
    pub translation: [f32; 3],
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct SpawnFlag {
    pub entity: Entity,
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct DropFlag {
    pub entity: Entity,
    pub translation: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct PickFlag {
    pub flag: Entity,
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct SpawnUnit {
    pub owner: Owner,
    pub entity: Entity,
    pub unit_type: UnitType,
    pub translation: [f32; 3],
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct SpawnProjectile {
    pub entity: Entity,
    pub projectile_type: ProjectileType,
    pub translation: [f32; 3],
    pub direction: [f32; 2],
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct BuildingUpdate {
    pub indicator: SceneBuildingIndicator,
    pub update: UpdateType,
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct LoadBuilding {
    pub indicator: SceneBuildingIndicator,
    pub status: BuildStatus,
    pub upgrade: Building,
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub enum UpdateType {
    Status { new_status: BuildStatus },
    Upgrade { upgraded_building: Building },
}

#[derive(Debug, Serialize, Deserialize, Component, Clone, PartialEq, Eq, Copy)]
pub enum Checkbox {
    Checked,
    Unchecked,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    PlayerJoinedLobby {
        id: ClientId,
        ready_state: Checkbox,
    },
    PlayerLeftLobby {
        id: ClientId,
    },
    LobbyPlayerReadyState {
        id: ClientId,
        ready_state: Checkbox,
    },
    SpawnPlayer(SpawnPlayer),
    SpawnFlag(SpawnFlag),
    DropFlag(DropFlag),
    PickFlag(PickFlag),
    SpawnUnit(SpawnUnit),
    SpawnProjectile(SpawnProjectile),
    PlayerDisconnected {
        id: ClientId,
    },
    DespawnEntity {
        entities: Vec<Entity>,
    },
    LoadGameScene {
        game_scene_type: GameSceneType,
        players: Vec<SpawnPlayer>,
        flag: Option<SpawnFlag>,
        units: Vec<SpawnUnit>,
        projectiles: Vec<SpawnProjectile>,
        buildings: Vec<LoadBuilding>,
    },
    SpawnGroup {
        player: SpawnPlayer,
        units: Vec<SpawnUnit>,
    },
    MeleeAttack {
        entity: Entity,
    },
    SyncInventory(Inventory),
    BuildingUpdate(BuildingUpdate),
    EntityHit {
        entity: Entity,
    },
    EntityDeath {
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
