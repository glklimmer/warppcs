use bevy::prelude::*;

use health::HealthPlugin;

use crate::networking::UnitType;

pub mod health;

#[derive(Component, Clone)]
pub struct Unit {
    pub unit_type: UnitType,
    pub swing_timer: Timer,
}

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HealthPlugin);
    }
}
