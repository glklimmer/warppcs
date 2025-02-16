use bevy::prelude::*;

use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};

use crate::{
    enum_map::*,
    physics::collider::{horse_collider, projectile_collider, BoxCollider},
};

pub mod chest;
pub mod spawn_point;

#[derive(Component)]
pub struct DelayedDespawn(Timer);

#[derive(Component, Debug, Serialize, Deserialize, Clone, Copy, Mappable)]
pub enum UnitType {
    Shieldwarrior,
    Pikeman,
    Archer,
    Bandit,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone, Copy)]
#[require(BoxCollider(horse_collider))]
pub enum MountType {
    Horse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: &'static str,
    pub tooltip: &'static str,
    pub effects_unit: UnitType,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Faction {
    Player { client_id: ClientId },
    Bandits,
}

#[derive(Debug, Component, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct Owner {
    pub faction: Faction,
}

#[derive(Debug, Component, PartialEq, Serialize, Deserialize, Copy, Clone)]
#[require(BoxCollider(projectile_collider))]
pub enum ProjectileType {
    Arrow,
}
