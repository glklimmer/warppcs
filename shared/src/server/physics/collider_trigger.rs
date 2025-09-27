use bevy::{math::bounding::IntersectsVolume, prelude::*};

use crate::{
    BoxCollider, Player,
    server::{
        game_scenes::travel::Traveling,
        players::interaction::{InteractionTriggeredEvent, InteractionType},
    },
};

#[derive(Component)]
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
    players: Query<Entity, (With<Player>, Without<Traveling>)>,
    triggers: Query<(Entity, &ColliderTrigger, &Transform, &BoxCollider)>,
    player_query: Query<(&Transform, &BoxCollider)>,
    mut interaction: EventWriter<InteractionTriggeredEvent>,
) {
    for player in players.iter() {
        let (player_transform, player_collider) = player_query.get(player).unwrap();
        let player_bounds = player_collider.at(player_transform);

        for (entity, trigger, transform, collider) in triggers.iter() {
            if !(player_bounds.intersects(&collider.at(transform))) {
                continue;
            }
            match trigger {
                ColliderTrigger::Travel => interaction.write(InteractionTriggeredEvent {
                    player,
                    interactable: entity,
                    interaction: InteractionType::Travel,
                }),
            };
        }
    }
}
