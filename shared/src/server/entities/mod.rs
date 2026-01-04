use bevy::prelude::*;

use enum_mappable::Mappable;

use crate::enum_map::EnumIter;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum UnitAnimation {
    #[default]
    Idle,
    Walk,
    Attack,
    Hit,
    Death,
}
