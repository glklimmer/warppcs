use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use bevy_renet::renet::RenetServer;

use crate::map::GameSceneId;
use crate::networking::{
    Facing, MultiplayerRoles, Owner, ProjectileType, ServerChannel, ServerMessages,
};
use crate::server::ai::attack::projectile_damage;
use crate::server::entities::health::TakeDamage;
use crate::{BoxCollider, DelayedDespawn, GameState};

use super::movement::Velocity;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            projectile_collision
                .run_if(in_state(GameState::GameSession).and(in_state(MultiplayerRoles::Host))),
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
        if transform.translation.y - collider.dimension.y <= 0.0 {
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
            let projectile = collider.at(transform);
            let target = target_collider.at(target_transform);

            if projectile.intersects(&target) {
                let delta_x = target_transform.translation.x - transform.translation.x;

                attack_events.send(TakeDamage {
                    target_entity,
                    damage: projectile_damage(projectile_type),
                    direction: match delta_x > 0. {
                        true => Facing::Left,
                        false => Facing::Right,
                    },
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
