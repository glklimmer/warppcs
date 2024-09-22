use bevy::prelude::*;

use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy_renet::renet::RenetServer;

use shared::networking::{Owner, ProjectileType, ServerChannel, ServerMessages};
use shared::BoxCollider;

use crate::ai::attack::{projectile_damage, TakeDamage};

use super::movement::Velocity;

#[derive(Component)]
struct DelayedDespawn(Timer);

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, projectile_collision);
        app.add_systems(PostUpdate, delayed_despawn);
    }
}

fn projectile_collision(
    mut commands: Commands,
    mut projectiles: Query<(
        Entity,
        &Transform,
        &mut Velocity,
        &BoxCollider,
        &ProjectileType,
        &Owner,
    )>,
    targets: Query<(Entity, &Transform, &BoxCollider, &Owner), Without<ProjectileType>>,
    mut server: ResMut<RenetServer>,
    mut attack_events: EventWriter<TakeDamage>,
) {
    for (entity, transform, mut velocity, collider, projectile_type, owner) in &mut projectiles {
        if transform.translation.y - collider.0.y <= 0.0 {
            velocity.0 = Vec2::ZERO;
            commands.entity(entity).remove::<BoxCollider>();
            commands
                .entity(entity)
                .insert(DelayedDespawn(Timer::from_seconds(3., TimerMode::Once)));
            continue;
        }

        for (target_entity, target_transform, target_collider, target_owner) in targets.iter() {
            if owner == target_owner {
                continue;
            }
            let projectile = Aabb2d::new(transform.translation.truncate(), collider.half_size());
            let target = Aabb2d::new(
                target_transform.translation.truncate(),
                target_collider.half_size(),
            );

            if projectile.intersects(&target) {
                attack_events.send(TakeDamage {
                    target_entity,
                    damage: projectile_damage(projectile_type),
                });
                commands.entity(entity).despawn();
                let despawn = ServerMessages::DespawnEntity { entity };
                let message = bincode::serialize(&despawn).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }
}

fn delayed_despawn(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DelayedDespawn)>,
    mut server: ResMut<RenetServer>,
    time: Res<Time>,
) {
    for (entity, mut delayed) in &mut query {
        let timer = &mut delayed.0;
        timer.tick(time.delta());

        if timer.just_finished() {
            commands.entity(entity).despawn();

            let despawn = ServerMessages::DespawnEntity { entity };
            let message = bincode::serialize(&despawn).unwrap();
            server.broadcast_message(ServerChannel::ServerMessages, message);
        }
    }
}
