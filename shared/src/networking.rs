use bevy::prelude::*;
use bevy_replicon::prelude::*;

use super::enum_map::*;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};

use crate::{
    horse_collider,
    map::{
        buildings::{BuildStatus, Building},
        GameSceneType,
    },
    projectile_collider, BoxCollider, Owner,
};

pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
}

pub struct NetworkRegistry;

impl Plugin for NetworkRegistry {
    fn build(&self, app: &mut App) {
        app.add_client_event::<LobbyEvent>(ChannelKind::Ordered);
    }
}

#[derive(Debug, Deserialize, Event, Serialize)]
pub enum LobbyEvent {
    StartGame,
    Ready(Checkbox),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Mappable)]
pub enum UnitType {
    Shieldwarrior,
    Pikeman,
    Archer,
    Bandit,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone, Copy)]
#[require(BoxCollider(horse_collider))]
pub enum MountType {
    Horse,
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub enum PlayerCommand {
    StartGame,
    Interact,
    MeleeAttack,
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
        Self { gold: 600 }
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Mounted {
    pub mount_type: MountType,
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct SpawnPlayer {
    pub id: ClientId,
    pub entity: Entity,
    pub translation: [f32; 3],
    pub mounted: Option<Mounted>,
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct SpawnFlag {
    pub flag: Entity,
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct DropFlag {
    pub flag: Entity,
    pub translation: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub struct PickFlag {
    pub flag: Entity,
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
    PlayerDisconnected {
        id: ClientId,
    },
    DespawnEntity {
        entities: Vec<Entity>,
    },
    MeleeAttack {
        entity: Entity,
    },
    SyncInventory(Inventory),
    EntityHit {
        entity: Entity,
    },
    EntityDeath {
        entity: Entity,
    },
    PlayerDefeat(Owner),
    Mount {
        entity: Entity,
        mount_type: MountType,
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

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::NetworkedEntities => 0,
            ServerChannel::ServerMessages => 1,
        }
    }
}
