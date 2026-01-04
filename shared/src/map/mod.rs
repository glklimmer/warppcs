use bevy::prelude::*;

use enum_as_f32_macro::enum_as_f32;

#[enum_as_f32]
#[derive(Component)]
pub enum Layers {
    Background,
    Building,
    Chest,
    Mount,
    Unit,
    Projectile,
    Flag,
    Item,
    Player,
    Wall,
    UI,
}
