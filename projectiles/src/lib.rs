use bevy::prelude::*;

use bevy::{math::bounding::IntersectsVolume, sprite::Anchor};
use bevy_replicon::prelude::Replicated;
use health::{Health, TakeDamage};
use physics::{collider::BoxCollider, movement::Velocity};
use serde::{Deserialize, Serialize};
use shared::{DelayedDespawn, Hitby, Owner, projectile_collider};
use units::Damage;

#[derive(Debug, Component, PartialEq, Serialize, Deserialize, Copy, Clone)]
#[require(Replicated, Velocity, Transform, BoxCollider = projectile_collider(), Sprite, Anchor::BOTTOM_CENTER)]
pub enum ProjectileType {
    Arrow,
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.replicate_bundle::<(ProjectileType, Transform)>()
            .add_systems(FixedUpdate, projectile_collision);
    }
}

type TargetComponents<'a> = (Entity, &'a Transform, &'a BoxCollider, &'a Owner);

#[allow(clippy::type_complexity)]
fn projectile_collision(
    mut commands: Commands,
    mut projectiles: Query<
        (
            Entity,
            &Transform,
            &mut Velocity,
            &BoxCollider,
            &Owner,
            &Damage,
        ),
        With<ProjectileType>,
    >,
    targets: Query<TargetComponents, (With<Health>, Without<ProjectileType>)>,
    mut attack_events: MessageWriter<TakeDamage>,
) {
    for (entity, transform, mut velocity, collider, owner, damage) in &mut projectiles {
        if transform.translation.y - collider.dimension.y <= 0.0 {
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

            let projectile = collider.at(transform);
            let target = target_collider.at(target_transform);

            if projectile.intersects(&target) {
                let delta_x = target_transform.translation.x - transform.translation.x;

                attack_events.write(TakeDamage {
                    target_entity,
                    damage: **damage,
                    direction: delta_x.into(),
                    by: Hitby::Arrow,
                });
                commands.entity(entity).despawn();
            }
        }
    }
}
