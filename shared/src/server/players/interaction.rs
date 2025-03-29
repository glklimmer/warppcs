use bevy::prelude::*;
use bevy_replicon::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use serde::{Deserialize, Serialize};

use crate::{BoxCollider, Faction, Owner, PhysicalPlayer};

#[derive(Clone, Copy, Debug)]
pub enum InteractionType {
    Chest,
    Mount,
    Travel,
    Building,
    Recruit,
    Flag,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Interactable {
    pub kind: InteractionType,
    pub restricted_to: Option<Owner>,
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
        app.add_client_trigger::<Interact>(ChannelKind::Ordered)
            .add_observer(interact)
            .add_event::<InteractionTriggeredEvent>()
            .add_systems(PostUpdate, send_interact.before(ClientSet::Send));
    }
}

#[derive(Event, Serialize, Deserialize, Debug)]
struct Interact;

impl InteractionType {
    pub fn priority(&self) -> i32 {
        match self {
            Self::Chest => 140,
            Self::Travel => 130,
            Self::Mount => 125,
            Self::Building => 122,
            Self::Flag => 120,
            Self::Recruit => 96,
        }
    }
}

fn send_interact(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        commands.client_trigger(Interact);
    }
}

fn interact(
    trigger: Trigger<FromClient<Interact>>,
    mut triggered_events: EventWriter<InteractionTriggeredEvent>,
    players: Query<(Entity, &PhysicalPlayer, &Transform, &BoxCollider)>,
    interactables: Query<(Entity, &Transform, &BoxCollider, &Interactable)>,
) {
    for (entity, player, player_transform, player_collider) in &players {
        if trigger.client_id != **player {
            continue;
        }

        let player_bounds = player_collider.at(player_transform);

        let priority_interaction = interactables
            .iter()
            .filter(|(.., transform, collider, _)| {
                player_bounds.intersects(&collider.at(transform))
            })
            .filter(|(.., interactable)| match interactable.restricted_to {
                Some(owner) => match *owner {
                    Faction::Player(item_owner) => item_owner.eq(&entity),
                    Faction::Bandits => false,
                },
                None => true,
            })
            .max_by(
                |(.., transform_a, _, interactable_a), (.., transform_b, _, interactable_b)| {
                    let priority_a = interactable_a.kind.priority();
                    let priority_b = interactable_b.kind.priority();
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
                player: entity,
                interactable,
                interaction: interaction.kind,
            });
        }
    }
}
