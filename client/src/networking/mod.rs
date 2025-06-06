use avian2d::prelude::Collider;
use bevy::prelude::*;

use shared::{BoxCollider, player_box_collider, player_collider};

use crate::animations::king::KingAnimation;

pub mod join_server;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Connected;

#[derive(Component)]
#[require(BoxCollider = player_box_collider(), Collider = player_collider(), KingAnimation)]
pub struct ControlledPlayer;
