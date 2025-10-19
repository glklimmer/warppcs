use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, Vec3LayerExt,
    map::Layers,
    networking::MountType,
    server::{game_scenes::GameSceneId, physics::movement::Velocity},
    unit_collider,
};

use super::{
    interaction::{Interactable, InteractionTriggeredEvent, InteractionType},
    items::Item,
};

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = unit_collider(),
    Velocity,
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    Interactable {
        kind: InteractionType::Mount,
        restricted_to: None,
    },
)]
pub struct Mount {
    pub mount_type: MountType,
}

#[derive(Component, Clone, Copy, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = chest_collider(),
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    Interactable{
        kind: InteractionType::Chest,
        restricted_to: None,
    },
)]
pub enum Chest {
    Normal,
    Big,
}

#[derive(Component, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ChestOpened;

pub fn open_chest(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    query: Query<(&Transform, &GameSceneId)>,
) {
    for event in interactions.read() {
        let InteractionType::Chest = &event.interaction else {
            continue;
        };

        commands
            .entity(event.interactable)
            .insert(ChestOpened)
            .remove::<Interactable>();

        let (chest_transform, game_scene_id) = query.get(event.interactable).unwrap();
        let chest_translation = chest_transform.translation;

        for _ in 0..3 {
            let item = Item::random();
            commands.spawn((
                item.collider(),
                item,
                *game_scene_id,
                chest_translation.with_y(12.5).with_layer(Layers::Item),
                Velocity(Vec2::new((fastrand::f32() - 0.5) * 50., 50.)),
            ));
        }
    }
}

fn chest_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 10.),
        offset: Some(Vec2::new(0., 5.)),
    }
}
