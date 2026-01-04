use bevy::prelude::*;

use serde::{Deserialize, Serialize};
use shared::BoxCollider;

#[derive(Component, Debug, Serialize, Deserialize, Clone, Copy)]
#[require(BoxCollider = horse_collider())]
pub enum MountType {
    Horse,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Mounted {
    pub mount_type: MountType,
}

fn horse_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 16.),
        offset: Some(Vec2::new(0., 8.)),
    }
}
