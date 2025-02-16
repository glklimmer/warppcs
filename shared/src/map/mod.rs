use bevy::prelude::*;

use enum_as_f32_macro::enum_as_f32;

pub mod buildings;
pub mod scenes;
pub mod spawn_point;

#[enum_as_f32]
#[derive(Component)]
pub enum Layers {
    Background,
    Building,
    Chest,
    Unit,
    Projectile,
    Flag,
    Player,
    Wall,
}
