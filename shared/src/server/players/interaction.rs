use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_replicon::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use serde::{Deserialize, Serialize};

use crate::{BoxCollider, ClientPlayerMap, Faction, Highlightable, Owner};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum InteractionType {
    Recruit,
    Flag,
    Building,
    Mount,
    Travel,
    Chest,
    Item,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
#[require(Replicated, Highlightable)]
pub struct Interactable {
    pub kind: InteractionType,
    pub restricted_to: Option<Owner>,
}

#[derive(Event, Clone, Copy, Serialize, Deserialize)]
pub struct InteractableSound {
    pub kind: InteractionType,
}

impl MapEntities for Interactable {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        match self.restricted_to {
            Some(owner) => match *owner {
                Faction::Player(entity) => {
                    self.restricted_to =
                        Some(Owner(Faction::Player(entity_mapper.map_entity(entity))));
                }
                Faction::Bandits => (),
            },
            None => (),
        }
    }
}

#[derive(Event)]
pub struct InteractionTriggeredEvent {
    pub player: Entity,
    pub interactable: Entity,
    pub interaction: InteractionType,
}

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_trigger::<Interact>(Channel::Ordered)
            .add_observer(interact)
            .add_event::<InteractionTriggeredEvent>()
            .add_systems(PostUpdate, send_interact.before(ClientSet::Send));
    }
}

#[derive(Event, Serialize, Deserialize, Debug)]
struct Interact;

fn send_interact(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        commands.client_trigger(Interact);
    }
}

fn interact(
    trigger: Trigger<FromClient<Interact>>,
    mut triggered_events: EventWriter<InteractionTriggeredEvent>,
    players: Query<(&Transform, &BoxCollider)>,
    interactables: Query<(Entity, &Transform, &BoxCollider, &Interactable)>,
    client_player_map: Res<ClientPlayerMap>,
) {
    let player = *client_player_map.get(&trigger.client_entity).unwrap();
    let (player_transform, player_collider) = players.get(player).unwrap();

    let player_bounds = player_collider.at(player_transform);

    let priority_interaction = interactables
        .iter()
        .filter(|(.., transform, collider, _)| player_bounds.intersects(&collider.at(transform)))
        .filter(|(.., interactable)| match interactable.restricted_to {
            Some(owner) => match *owner {
                Faction::Player(item_owner) => item_owner.eq(&player),
                Faction::Bandits => false,
            },
            None => true,
        })
        .max_by(
            |(.., transform_a, _, interactable_a), (.., transform_b, _, interactable_b)| {
                let priority_a = interactable_a.kind as i32;
                let priority_b = interactable_b.kind as i32;
                if priority_a == priority_b {
                    let distance_a = player_transform
                        .translation
                        .distance(transform_a.translation);
                    let distance_b = player_transform
                        .translation
                        .distance(transform_b.translation);
                    distance_b.total_cmp(&distance_a)
                } else {
                    priority_a.cmp(&priority_b)
                }
            },
        );

    if let Some((interactable, _, _, interaction)) = priority_interaction {
        triggered_events.send(InteractionTriggeredEvent {
            player,
            interactable,
            interaction: interaction.kind,
        });
    }
}
