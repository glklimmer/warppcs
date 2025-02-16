use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{entities::MountType, networking::Checkbox};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub enum PlayerCommand {
    StartGame,
    Interact,
    MeleeAttack,
    LobbyReadyState(Checkbox),
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
