use bevy::prelude::*;

use shared::{BoxCollider, player_collider};

use crate::animations::king::KingAnimation;

pub mod join_server;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Connected;

#[derive(Component)]
#[require(BoxCollider(player_collider), KingAnimation)]
pub struct ControlledPlayer;
