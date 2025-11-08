use bevy::prelude::*;
use bevy_replicon::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use serde::{Deserialize, Serialize};

use crate::{BoxCollider, ClientPlayerMap, ClientPlayerMapExt, PlayerState};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum InteractionType {
    Unmount,
    Recruit,
    Flag,
    ItemAssignment,
    Building,
    Travel,
    Portal,
    Mount,
    Commander,
    Chest,
    Item,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
#[require(Replicated)]
pub struct Interactable {
    pub kind: InteractionType,
    #[entities]
    pub restricted_to: Option<Entity>,
}

#[derive(Event, Clone, Copy, Serialize, Deserialize)]
pub struct InteractableSound {
    pub kind: InteractionType,
    pub spatial_position: Vec3,
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
            .add_systems(
                PostUpdate,
                send_interact
                    .before(ClientSet::Send)
                    .run_if(in_state(PlayerState::World)),
            );
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
) -> Result {
    let player = *client_player_map.get_player(&trigger.client_entity)?;
    let (player_transform, player_collider) = players.get(player)?;

    let player_bounds = player_collider.at(player_transform);

    let priority_interaction = interactables
        .iter()
        .filter(|(.., transform, collider, _)| player_bounds.intersects(&collider.at(transform)))
        .filter(|(.., interactable)| match interactable.restricted_to {
            Some(owner) => owner.eq(&player),
            None => true,
        })
        .max_by(
            |(.., transform_a, _, interactable_a), (.., transform_b, _, interactable_b)| {
                let priority_a = interactable_a.kind as i32;
                let priority_b = interactable_b.kind as i32;

                if priority_a != priority_b {
                    return priority_a.cmp(&priority_b);
                }

                let distance_a = player_transform
                    .translation
                    .distance(transform_a.translation);
                let distance_b = player_transform
                    .translation
                    .distance(transform_b.translation);
                distance_b.total_cmp(&distance_a)
            },
        );

    if let Some((interactable, .., interaction)) = priority_interaction {
        triggered_events.write(InteractionTriggeredEvent {
            player,
            interactable,
            interaction: interaction.kind,
        });
    }
    Ok(())
}
