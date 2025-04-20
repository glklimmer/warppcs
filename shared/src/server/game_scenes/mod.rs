use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};

use crate::server::players::interaction::InteractionType;

use super::{
    buildings::recruiting::{FlagAssignment, FlagHolder},
    entities::Unit,
    players::interaction::{Interactable, InteractionTriggeredEvent},
};
use crate::{BoxCollider, map::Layers};

pub mod start_game;

#[derive(Component, Clone, Deref)]
pub struct TravelDestination(Entity);

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider(portal_collider),
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
    Interactable(|| Interactable {
        kind: InteractionType::Travel,
        restricted_to: None,
    }),
)]
pub struct Portal;

fn portal_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(32., 32.),
        offset: Some(Vec2::new(0., 16.)),
    }
}

pub struct GameScenesPlugin;

impl Plugin for GameScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, travel);
    }
}

fn travel(
    mut commands: Commands,
    mut traveling: EventReader<InteractionTriggeredEvent>,
    flag_holders: Query<&FlagHolder>,
    units_on_flag: Query<(Entity, &FlagAssignment, &Unit)>,
    destination: Query<&TravelDestination>,
    transform: Query<&Transform>,
) {
    for event in traveling.read() {
        let InteractionType::Travel = &event.interaction else {
            continue;
        };

        let player_entity = event.player;
        let destination = destination.get(event.interactable).unwrap();
        let target_position = transform.get(**destination).unwrap().translation;
        info!("Travel happening to target position: {:?}", target_position);

        let group = match flag_holders.get(player_entity) {
            Ok(flag_holder) => Some(FlagGroup {
                flag: flag_holder.0,
                units: units_on_flag
                    .iter()
                    .filter(|(_, assignment, _)| assignment.0.eq(&flag_holder.0))
                    .collect(),
            }),
            Err(_) => None,
        };

        commands
            .entity(player_entity)
            .insert(Transform::from_translation(
                target_position.with_z(Layers::Player.as_f32()),
            ));
        if let Some(group) = &group {
            commands.entity(group.flag).insert(
                Transform::from_translation(target_position.with_z(Layers::Unit.as_f32()))
                    .with_scale(Vec3::splat(1. / 3.)),
            );
            for (unit, _, _) in &group.units {
                commands.entity(*unit).insert(Transform::from_translation(
                    target_position.with_z(Layers::Flag.as_f32()),
                ));
            }
        }
    }
}
