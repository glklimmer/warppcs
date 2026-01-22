use bevy::prelude::*;
use bevy_replicon::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use lobby::{ClientPlayerMap, ClientPlayerMapExt};
use physics::movement::BoxCollider;
use serde::{Deserialize, Serialize};
use shared::PlayerState;

use crate::{collider_trigger::ColliderTriggerPlugin, sound::InteractionSoundPlugin};

pub mod collider_trigger;
pub mod sound;

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ColliderTriggerPlugin, InteractionSoundPlugin))
            .replicate::<Interactable>()
            .add_client_event::<Interact>(Channel::Ordered)
            .add_observer(interact)
            .add_message::<InteractionTriggeredEvent>()
            .add_systems(
                PostUpdate,
                send_interact
                    .before(ClientSystems::Send)
                    .run_if(in_state(PlayerState::World)),
            );
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum InteractionType {
    Unmount,
    Recruit,
    Flag,
    ItemAssignment,
    Building,
    Travel,
    Portal,
    Collect,
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

#[derive(Message)]
pub struct InteractionTriggeredEvent {
    pub player: Entity,
    pub interactable: Entity,
    pub interaction: InteractionType,
}

#[derive(Component)]
pub struct ActiveInteraction {
    pub interactable: Entity,
}

#[derive(Event, Serialize, Deserialize, Debug)]
struct Interact(usize);

fn send_interact(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        commands.client_trigger(Interact(0));
    }
}

fn interact(
    trigger: On<FromClient<Interact>>,
    mut triggered_events: MessageWriter<InteractionTriggeredEvent>,
    players: Query<(&Transform, &BoxCollider)>,
    interactables: Query<(Entity, &Transform, &BoxCollider, &Interactable)>,
    client_player_map: Res<ClientPlayerMap>,
) -> Result {
    let player = *client_player_map.get_player(&trigger.client_id)?;
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
