use bevy::prelude::*;
use bevy_replicon::prelude::server_or_singleplayer;
use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, GRAVITY_G, Owner,
    map::buildings::{BuildStatus, Building},
    server::{
        ai::UnitBehaviour,
        entities::{Unit, health::Health},
    },
};
use bevy::math::bounding::IntersectsVolume;

use super::{PushBack, projectile::ProjectileType};

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct Velocity(pub Vec2);

#[derive(Component, Debug, Copy, Clone, Deref)]
pub struct Speed(pub f32);

#[derive(Component, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Grounded;

#[derive(Component, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Moving;

impl Default for Speed {
    fn default() -> Self {
        Self(70.0)
    }
}

#[derive(Component)]
pub struct RandomVelocityMul(f32);

impl Default for RandomVelocityMul {
    fn default() -> Self {
        Self(fastrand::choice([0.9, 0.95, 1.0, 1.1, 1.15]).unwrap())
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                (
                    set_unit_velocity,
                    set_grounded,
                    set_walking,
                    apply_friction,
                    set_projectile_rotation,
                ),
                wall_collision,
            )
                .chain()
                .run_if(server_or_singleplayer),
        );
        app.add_systems(
            FixedPostUpdate,
            (apply_gravity, apply_velocity)
                .chain()
                .run_if(server_or_singleplayer),
        );
    }
}

fn apply_gravity(mut query: Query<(&mut Velocity, &Transform, &BoxCollider)>, time: Res<Time>) {
    for (mut velocity, transform, collider) in &mut query {
        let bottom = transform.translation.truncate() - collider.half_size()
            + collider.offset.unwrap_or_default();
        let next_bottom = bottom.y + velocity.0.y * time.delta_secs();

        if next_bottom > 0. {
            velocity.0.y -= GRAVITY_G * time.delta_secs();
        } else if velocity.0.y < 0. {
            velocity.0.y = 0.;
        }
    }
}

fn apply_velocity(
    mut query: Query<(&Velocity, &mut Transform), Changed<Velocity>>,
    time: Res<Time>,
) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.) * time.delta_secs();
    }
}

fn apply_friction(mut query: Query<&mut Velocity, With<Grounded>>, time: Res<Time>) {
    let friction_force = 400.0 * time.delta_secs();
    for mut velocity in query.iter_mut() {
        if velocity.0.x.abs() <= friction_force {
            velocity.0.x = 0.0;
        } else {
            velocity.0.x -= velocity.0.x.signum() * friction_force;
        }
    }
}

fn set_grounded(mut commands: Commands, entities: Query<(Entity, &Transform)>) {
    for (entity, transform) in &entities {
        let Some(mut entity) = commands.get_entity(entity) else {
            continue;
        };

        if transform.translation.y == 0. {
            entity.try_insert(Grounded);
        } else {
            entity.remove::<Grounded>();
        }
    }
}

fn set_walking(mut commands: Commands, entities: Query<(Entity, &Velocity, Option<&Grounded>)>) {
    for (entity, velocity, maybe_grounded) in &entities {
        let Some(mut entity) = commands.get_entity(entity) else {
            continue;
        };

        if maybe_grounded.is_some() && velocity.0.x.abs() > 0. {
            entity.try_insert(Moving);
        } else {
            entity.remove::<Moving>();
        }
    }
}

fn set_projectile_rotation(
    mut projectiles: Query<(&mut Transform, &Velocity), With<ProjectileType>>,
) {
    for (mut transform, velocity) in projectiles.iter_mut() {
        let angle = velocity.0.to_angle();
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

fn wall_collision(
    mut query: Query<(&mut Velocity, &Transform, &BoxCollider, &Owner)>,
    buildings: Query<(&Transform, &BoxCollider, &Owner, &Building, &BuildStatus), With<Health>>,
    time: Res<Time>,
) {
    for (mut velocity, transform, collider, owner) in query.iter_mut() {
        let future_position = transform.translation.truncate() + velocity.0 * time.delta_secs();
        let future_bounds = collider.at_pos(future_position);

        for (building_transform, building_collider, building_owner, building, building_status) in
            buildings.iter()
        {
            if building_owner.eq(owner) {
                continue;
            }

            if building_status.ne(&BuildStatus::Built) {
                continue;
            }
            if let Building::Wall { level: _ } = building {
                let building_bounds = building_collider.at(building_transform);

                if building_bounds.intersects(&future_bounds) {
                    velocity.0.x = 0.;
                    break;
                }
            }
        }
    }
}

const MOVE_EPSILON: f32 = 1.;

#[allow(clippy::type_complexity)]
fn set_unit_velocity(
    mut query: Query<
        (
            &mut Velocity,
            &mut Transform,
            &UnitBehaviour,
            &PushBack,
            &RandomVelocityMul,
            &Speed,
        ),
        (With<Unit>, With<Health>),
    >,
    transform_query: Query<&Transform, Without<Unit>>,
) {
    for (mut velocity, mut transform, behaviour, push_back, rand_velocity_mul, speed) in &mut query
    {
        match behaviour {
            UnitBehaviour::Idle => {}
            UnitBehaviour::AttackTarget(_) => {
                if !push_back.timer.finished() {
                    continue;
                }
                velocity.0.x = 0.;
            }
            UnitBehaviour::FollowFlag(flag, offset) => {
                let target = transform_query.get(*flag).unwrap().translation.truncate();
                let target = target + *offset;
                let direction = (target.x - transform.translation.x).signum();

                if (transform.translation.x - target.x).abs() <= MOVE_EPSILON {
                    velocity.0.x = 0.;
                    continue;
                }

                velocity.0.x = direction * **speed * rand_velocity_mul.0;
                transform.scale.x = direction;
            }
        }
    }
}
