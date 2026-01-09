use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{AppRuleExt, Replicated};
use interaction::{Interactable, InteractionType};
use physics::movement::BoxCollider;
use serde::{Deserialize, Serialize};

use crate::portal::animation::PortalAnimationPlugin;

mod animation;

pub(crate) struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PortalAnimationPlugin)
            .replicate_bundle::<(Portal, Transform)>()
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = portal_collider(),
    Sprite,
    Anchor::BOTTOM_CENTER,
    Interactable {
        kind: InteractionType::Portal,
        restricted_to: None,
    },
)]
pub struct Portal;

fn portal_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(32., 32.),
        offset: Some(Vec2::new(0., 16.)),
    }
}

#[derive(Component, Deref)]
pub struct PortalDestination(Entity);
