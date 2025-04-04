use bevy::prelude::*;
use bevy_replicon::prelude::*;

use super::enum_map::*;
use serde::{Deserialize, Serialize};

use crate::{BoxCollider, horse_collider};

pub const PROTOCOL_ID: u64 = 7;

pub struct NetworkRegistry;

impl Plugin for NetworkRegistry {
    fn build(&self, app: &mut App) {
        app.add_client_event::<LobbyEvent>(Channel::Ordered);
    }
}

#[derive(Debug, Deserialize, Event, Serialize)]
pub enum LobbyEvent {
    StartGame,
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
}

pub enum ClientChannel {
    Input,
    Command,
}
pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[require(Replicated)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum Facing {
    #[default]
    Left,
    Right,
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
