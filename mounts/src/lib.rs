use bevy::{prelude::*, sprite::Anchor};

use bevy_replicon::prelude::{AppRuleExt, Replicated};
use interaction::{Interactable, InteractionType};
use physics::movement::{BoxCollider, Velocity};
use serde::{Deserialize, Serialize};

pub struct MountPlugins;

impl Plugin for MountPlugins {
    fn build(&self, app: &mut App) {
        app.replicate::<Mounted>()
            .replicate_bundle::<(Mount, Transform)>();
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = horse_collider(),
    Velocity,
    Sprite,
    Anchor::BOTTOM_CENTER,
    Interactable{
        kind: InteractionType::Mount,
        restricted_to: None,
    },
)]
pub struct Mount {
    pub mount_type: MountType,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone, Copy)]
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
