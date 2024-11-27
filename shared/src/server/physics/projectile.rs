use bevy::prelude::*;

use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy_renet::renet::RenetServer;

use crate::map::GameSceneId;
use crate::networking::{MultiplayerRoles, Owner, ProjectileType, ServerChannel, ServerMessages};
use crate::server::ai::attack::projectile_damage;
use crate::server::entities::health::TakeDamage;
use crate::{BoxCollider, GameState};

use super::movement::Velocity;

#[derive(Component)]
struct DelayedDespawn(Timer);

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            projectile_collision.run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
        app.add_systems(
            PostUpdate,
            delayed_despawn.run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
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
        &GameSceneId,
    )>,
    targets: Query<
        (Entity, &Transform, &BoxCollider, &Owner, &GameSceneId),
        Without<ProjectileType>,
    >,
    mut server: ResMut<RenetServer>,
    mut attack_events: EventWriter<TakeDamage>,
) {
    for (entity, transform, mut velocity, collider, projectile_type, owner, scene_id) in
        &mut projectiles
    {
        if transform.translation.y - collider.0.y <= 0.0 {
            velocity.0 = Vec2::ZERO;
            commands.entity(entity).remove::<BoxCollider>();
            commands
                .entity(entity)
                .insert(DelayedDespawn(Timer::from_seconds(3., TimerMode::Once)));
            continue;
        }

        for (target_entity, target_transform, target_collider, target_owner, target_scene_id) in
            targets.iter()
        {
            if owner == target_owner || scene_id != target_scene_id {
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
                let despawn = ServerMessages::DespawnEntity {
                    entities: vec![entity],
                };
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

            let despawn = ServerMessages::DespawnEntity {
                entities: vec![entity],
            };
            let message = bincode::serialize(&despawn).unwrap();
            server.broadcast_message(ServerChannel::ServerMessages, message);
        }
    }
}
