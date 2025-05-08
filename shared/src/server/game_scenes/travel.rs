use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};

use crate::{PlayerState, server::players::interaction::InteractionType};

use super::super::{
    buildings::recruiting::{FlagAssignment, FlagHolder},
    entities::{Unit, commander::SlotsAssignments},
    players::interaction::{Interactable, InteractionTriggeredEvent},
};
use crate::{BoxCollider, map::Layers};

pub struct TravelPlugin;

impl Plugin for TravelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, travel_timer)
            .add_systems(FixedUpdate, (start_travel, end_travel));
    }
}

#[derive(Component)]
struct Traveling {
    destination: TravelDestination,
    time_left: Timer,
}

impl Traveling {
    fn to(destination: TravelDestination) -> Self {
        Self {
            destination,
            time_left: Timer::from_seconds(1., TimerMode::Once),
        }
    }
}

#[derive(Component, Clone, Deref)]
pub struct TravelDestination(Entity);

impl TravelDestination {
    pub fn new(destination: Entity) -> Self {
        Self(destination)
    }
}

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

fn travel_timer(mut query: Query<&mut Traveling>, time: Res<Time>) {
    for mut traveling in &mut query {
        info!(
            "Travel happening, time left: {:?}",
            traveling.time_left.elapsed_secs()
        );
        traveling.time_left.tick(time.delta());
    }
}

#[allow(clippy::too_many_arguments)]
fn start_travel(
    mut commands: Commands,
    mut traveling: EventReader<InteractionTriggeredEvent>,
    flag_holders: Query<&FlagHolder>,
    commanders: Query<(&FlagAssignment, &SlotsAssignments)>,
    units_on_flag: Query<(Entity, &FlagAssignment, &Unit)>,
    destination: Query<&TravelDestination>,
    transform: Query<&Transform>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    for event in traveling.read() {
        let InteractionType::Travel = &event.interaction else {
            continue;
        };

        let player_entity = event.player;
        let destination = destination.get(event.interactable).unwrap();
        let target_position = transform.get(**destination).unwrap().translation;
        let flag_holder = flag_holders.get(player_entity);

        info!("Travel starting to target position: {:?}", target_position);

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
                        slots_assignments.slots.contains(&Some(assignment.0))
                    })
                    .for_each(|(entity, _, _)| travel_entities.push(entity));
            };
        };

        next_state.set(PlayerState::Traveling);

        commands
            .entity(player_entity)
            .insert(Traveling::to(destination.clone()));

        for group in travel_entities {
            commands
                .entity(group)
                .insert(Traveling::to(destination.clone()));
        }
    }
}

fn end_travel(
    mut commands: Commands,
    query: Query<(Entity, &Traveling)>,
    transform: Query<&Transform>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    for (entity, travel) in query.iter() {
        if !travel.time_left.finished() {
            continue;
        }

        let destination = &travel.destination;
        let travel_destination = transform.get(**destination).unwrap();
        let target_position = travel_destination.translation;

        info!("Travel finished to target position: {:?}", target_position);

        next_state.set(PlayerState::World);

        commands
            .entity(entity)
            .remove::<Traveling>()
            .insert(Transform::from_translation(
                target_position.with_z(Layers::Player.as_f32()),
            ));
    }
}
