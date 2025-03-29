use bevy::prelude::*;

use bevy_renet::renet::ClientId;
use shared::{networking::ServerMessages, player_collider, BoxCollider};

use crate::animations::king::KingAnimation;

pub mod join_server;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Connected;

#[derive(Component)]
#[require(BoxCollider(player_collider), KingAnimation)]
pub struct ControlledPlayer;

#[derive(Debug, Resource)]
pub struct CurrentClientId(pub ClientId);

// TODO: Remove this
#[derive(Event)]
pub struct NetworkEvent {
    pub message: ServerMessages,
}
