use bevy::prelude::*;

pub mod entities;
pub mod enum_map;
pub mod map;
pub mod networking;
pub mod physics;
pub mod player;
pub mod server;
pub mod steamworks;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    GameSession,
}
