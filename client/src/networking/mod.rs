use bevy::prelude::*;

pub mod join_server;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Connected;
