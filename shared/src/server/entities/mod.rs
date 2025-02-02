use bevy::prelude::*;

use health::HealthPlugin;

use crate::{
    networking::{MountType, UnitType},
    unit_collider, BoxCollider,
};

use super::physics::{movement::Velocity, PushBack};

pub mod health;

#[derive(Component, Clone)]
#[require(BoxCollider(unit_collider), Velocity, PushBack)]
pub struct Unit {
    pub unit_type: UnitType,
    pub swing_timer: Timer,
}

#[derive(Component, Clone)]
#[require(BoxCollider(unit_collider), Velocity)]
pub struct Mount {
    pub mount_type: MountType,
}

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HealthPlugin);
    }
}
