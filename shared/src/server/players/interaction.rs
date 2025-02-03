use bevy::{math::bounding::IntersectsVolume, prelude::*};

use bevy_renet::renet::ClientId;

use crate::{
    map::GameSceneId,
    networking::{Faction, Owner},
    server::networking::ServerLobby,
    BoxCollider,
};

use super::super::networking::NetworkEvent;

#[derive(Clone, Copy, Debug)]
pub enum InteractionType {
    Chest,
    Mount,
    Travel,
    Building,
    Recruit,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Interactable {
    pub kind: InteractionType,
    pub restricted_to: Option<Owner>,
}

#[derive(Event)]
pub struct InteractionTriggeredEvent {
    pub player: Entity,
    pub client_id: ClientId,
    pub interactable: Entity,
    pub interaction: InteractionType,
}

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractionTriggeredEvent>();

        app.add_systems(FixedUpdate, (interact).run_if(on_event::<NetworkEvent>));
    }
}

impl InteractionType {
    pub fn priority(&self) -> i32 {
        match self {
            Self::Chest => 100,
            Self::Mount => 99,
            Self::Travel => 98,
            Self::Building => 97,
            Self::Recruit => 1,
        }
    }
}

fn interact(
    mut network_events: EventReader<NetworkEvent>,
    mut triggered_events: EventWriter<InteractionTriggeredEvent>,
    lobby: Res<ServerLobby>,
    players: Query<(&Transform, &BoxCollider, &GameSceneId)>,
    interactables: Query<(
        Entity,
        &Transform,
        &BoxCollider,
        &GameSceneId,
        &Interactable,
    )>,
) {
    for event in network_events.read() {
        let client_id = event.client_id;
        let &player_entity = lobby.players.get(&client_id).unwrap();
        let Ok((player_transform, player_collider, player_scene)) = players.get(player_entity)
        else {
            continue;
        };

        let player_bounds = player_collider.at(player_transform);

        let priority_interaction = interactables
            .iter()
            .filter(|(_, _, _, scene, _)| player_scene.eq(*scene))
            .filter(|(_, transform, collider, _, _)| {
                player_bounds.intersects(&collider.at(transform))
            })
            .filter(
                |(_, _, _, _, interactable)| match interactable.restricted_to {
                    Some(owner) => match owner {
                        Owner {
                            faction:
                                Faction::Player {
                                    client_id: item_client_id,
                                },
                        } => item_client_id.eq(&client_id),
                        Owner {
                            faction: Faction::Bandits,
                        } => false,
                    },
                    None => true,
                },
            )
            .max_by_key(|(_, _, _, _, interactable)| interactable.kind.priority());

        if let Some((entity, _, _, portal_scene, interactable)) = priority_interaction {
            println!(
                "Sending interact for {:?}, from scene: {:?}",
                interactable.kind, portal_scene
            );
            triggered_events.send(InteractionTriggeredEvent {
                player: player_entity,
                client_id,
                interactable: entity,
                interaction: interactable.kind,
            });
        }
    }
}
