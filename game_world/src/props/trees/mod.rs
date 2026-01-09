use bevy::prelude::*;

use shared::enum_map::*;

pub(crate) mod pine;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub(crate) enum TreeAnimation {
    #[default]
    Windy,
}
