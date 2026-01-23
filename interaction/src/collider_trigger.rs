use bevy::{math::bounding::IntersectsVolume, prelude::*};

use crate::{ActiveInteraction, BoxCollider, InteractionTriggeredEvent, InteractionType};

#[derive(Component, Default)]
pub struct ColliderTriggerActivater;

#[derive(Component, PartialEq, Eq)]
pub enum ColliderTrigger {
    Travel,
}

pub struct ColliderTriggerPlugin;

impl Plugin for ColliderTriggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, check_collider_trigger);
    }
}

fn check_collider_trigger(
    activators: Query<Entity, (With<ColliderTriggerActivater>, Without<ActiveInteraction>)>,
    triggers: Query<(Entity, &ColliderTrigger, &Transform, &BoxCollider)>,
    player_query: Query<(&Transform, &BoxCollider)>,
    mut interaction: MessageWriter<InteractionTriggeredEvent>,
    mut commands: Commands,
) -> Result {
    for player in activators.iter() {
        let (player_transform, player_collider) = player_query.get(player)?;
        let player_bounds = player_collider.at(player_transform);

        for (entity, trigger, transform, collider) in triggers.iter() {
            if !(player_bounds.intersects(&collider.at(transform))) {
                continue;
            }

            commands.entity(player).insert(ActiveInteraction {
                interactable: entity,
            });

            match trigger {
                ColliderTrigger::Travel => interaction.write(InteractionTriggeredEvent {
                    player,
                    interactable: entity,
                    interaction: InteractionType::Travel,
                }),
            };
        }
    }
    Ok(())
}
