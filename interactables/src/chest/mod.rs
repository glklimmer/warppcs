use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{AppRuleExt, Replicated};
use interaction::{Interactable, InteractionTriggeredEvent, InteractionType};
use items::Item;
use physics::movement::{BoxCollider, Velocity};
use serde::{Deserialize, Serialize};
use shared::{GameSceneId, Vec3LayerExt, map::Layers};

mod animation;

pub(crate) struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<ChestOpened>()
            .replicate_bundle::<(Chest, Transform)>()
            .add_systems(
                FixedUpdate,
                open_chest.run_if(on_message::<InteractionTriggeredEvent>),
            );
    }
}

#[derive(Component, Clone, Copy, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = chest_collider(),
    Sprite,
    Anchor::BOTTOM_CENTER,
    Interactable {
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

pub(crate) fn open_chest(
    mut interactions: MessageReader<InteractionTriggeredEvent>,
    query: Query<(&Transform, &GameSceneId)>,
    mut commands: Commands,
) -> Result {
    for event in interactions.read() {
        let InteractionType::Chest = &event.interaction else {
            continue;
        };

        commands
            .entity(event.interactable)
            .insert(ChestOpened)
            .remove::<Interactable>();

        let (chest_transform, game_scene_id) = query.get(event.interactable)?;
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
    Ok(())
}

fn chest_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 10.),
        offset: Some(Vec2::new(0., 5.)),
    }
}
