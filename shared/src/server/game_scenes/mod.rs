use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};

use crate::server::players::interaction::InteractionType;

use super::{
    buildings::recruiting::{FlagAssignment, FlagHolder},
    entities::{Unit, commander::SlotsAssignments},
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
    commanders: Query<(&FlagAssignment, &SlotsAssignments)>,
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
        let flag_holder = flag_holders.get(player_entity);

        info!("Travel happening to target position: {:?}", target_position);

        let mut travel_entities = Vec::new();

        if let Ok(flag_holder) = flag_holder {
            units_on_flag
                .iter()
                .filter(|(_, assignment, _)| assignment.0 == flag_holder.0)
                .for_each(|(entity, _, _)| travel_entities.push(entity));

            let commander = commanders
                .iter()
                .find(|(assignment, _)| assignment.0.eq(&flag_holder.0));

            if let Some((_, slots_assignments)) = commander {
                units_on_flag
                    .iter()
                    .filter(|(_, assignment, _)| {
                        let possible_positions = vec![
                            slots_assignments.front,
                            slots_assignments.middle,
                            slots_assignments.back,
                        ];
                        possible_positions.contains(&Some(assignment.0))
                    })
                    .for_each(|(entity, _, _)| travel_entities.push(entity));
            };
        };

        commands
            .entity(player_entity)
            .insert(Transform::from_translation(
                target_position.with_z(Layers::Player.as_f32()),
            ));

        for group in travel_entities {
            commands.entity(group).insert(Transform::from_translation(
                target_position.with_z(Layers::Flag.as_f32()),
            ));
        }
    }
}
