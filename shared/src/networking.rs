use bevy::prelude::*;

use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_ID: u64 = 7;

pub struct NetworkRegistry;

impl Plugin for NetworkRegistry {
    fn build(&self, app: &mut App) {
        app.add_client_message::<LobbyMessage>(Channel::Ordered);
    }
}

#[derive(Debug, Deserialize, Message, Serialize)]
pub enum LobbyMessage {
    StartGame,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum WorldDirection {
    #[default]
    Left,
    Right,
}

impl From<f32> for WorldDirection {
    fn from(value: f32) -> Self {
        match value > 0. {
            true => WorldDirection::Right,
            false => WorldDirection::Left,
        }
    }
}

impl From<WorldDirection> for f32 {
    fn from(value: WorldDirection) -> Self {
        match value {
            WorldDirection::Left => -1.,
            WorldDirection::Right => 1.,
        }
    }
}
