use bevy::prelude::*;

use shared::enum_map::*;

pub mod trees;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum TreeAnimation {
    #[default]
    Windy,
}
