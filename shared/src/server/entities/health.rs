use bevy::prelude::*;

use bevy_renet::renet::RenetServer;

use crate::{
    networking::{MultiplayerRoles, ServerChannel, ServerMessages},
    GameState,
};

#[derive(Component, Clone)]
pub struct Health {
    pub hitpoints: f32,
}

#[derive(Event)]
pub struct TakeDamage {
    pub target_entity: Entity,
    pub damage: f32,
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamage>();

        app.add_systems(FixedUpdate, (apply_damage).run_if(on_event::<TakeDamage>()));

        app.add_systems(
            FixedUpdate,
            (on_unit_death).run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
    }
}

fn apply_damage(mut query: Query<&mut Health>, mut attack_events: EventReader<TakeDamage>) {
    for event in attack_events.read() {
        if let Ok(mut health) = query.get_mut(event.target_entity) {
            health.hitpoints -= event.damage;
            println!("New health: {}.", health.hitpoints);
        }
    }
}

fn on_unit_death(
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
    query: Query<(Entity, &Health)>,
) {
    for (entity, health) in query.iter() {
        if health.hitpoints <= 0. {
            commands.entity(entity).despawn_recursive();

            let message = ServerMessages::DespawnEntity {
                entities: vec![entity],
            };
            let unit_dead_message = bincode::serialize(&message).unwrap();
            server.broadcast_message(ServerChannel::ServerMessages, unit_dead_message);
        }
    }
}
