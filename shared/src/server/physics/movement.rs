use bevy::prelude::*;

use super::PushBack;
use crate::{
    map::{buildings::Building, GameSceneId},
    networking::{Owner, PlayerInput},
    server::{
        ai::{attack::unit_speed, UnitBehaviour},
        entities::{health::Health, Unit},
    },
    BoxCollider, GRAVITY_G,
};
use bevy::math::bounding::IntersectsVolume;

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct RandomVelocityMul(f32);

impl Default for RandomVelocityMul {
    fn default() -> Self {
        Self(fastrand::choice([0.9, 0.95, 1.0, 1.1, 1.15]).unwrap())
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Speed(pub f32);

impl Default for Speed {
    fn default() -> Self {
        Self(200.0)
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                apply_velocity,
                determine_unit_velocity,
                apply_gravity,
                move_players_system,
            ),
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

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.) * time.delta_secs();
    }
}

fn move_players_system(
    mut query: Query<(
        &mut Velocity,
        &PlayerInput,
        &Speed,
        &Transform,
        &BoxCollider,
        &GameSceneId,
        &Owner,
    )>,
    buildings: Query<(&Transform, &BoxCollider, &GameSceneId, &Owner, &Building), With<Health>>,
) {
    for (
        mut velocity,
        input,
        speed,
        player_transform,
        player_collider,
        player_scene,
        client_owner,
    ) in query.iter_mut()
    {
        let x = (input.right as i8 - input.left as i8) as f32;
        let direction = Vec2::new(x, 0.).normalize_or_zero();
        let desired_velocity = direction * speed.0;

        let future_position = player_transform.translation.truncate() + direction;
        let future_bounds = player_collider.at_pos(future_position);

        let mut would_collide = false;
        for (building_transform, building_collider, building_scene, owner, building) in
            buildings.iter()
        {
            if player_scene.ne(building_scene) {
                continue;
            }

            if owner.eq(client_owner) {
                continue;
            }

            if let Building::Wall { level: _ } = building {
                let building_bounds = building_collider.at(building_transform);

                if building_bounds.intersects(&future_bounds) {
                    would_collide = true;
                    break;
                }
            }
        }

        velocity.0 = if would_collide {
            Vec2::ZERO
        } else {
            desired_velocity
        };
    }
}

const MOVE_EPSILON: f32 = 1.;

fn determine_unit_velocity(
    mut query: Query<(
        &mut Velocity,
        &Transform,
        &UnitBehaviour,
        &Unit,
        &PushBack,
        &RandomVelocityMul,
    )>,
    transform_query: Query<&Transform>,
) {
    for (mut velocity, transform, behaviour, unit, push_back, rand_velocity_mul) in &mut query {
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

                if (transform.translation.x - target.x).abs() <= MOVE_EPSILON {
                    velocity.0.x = 0.;
                    continue;
                }

                let target_right = target.x > transform.translation.x;
                match target_right {
                    true => velocity.0.x = unit_speed(&unit.unit_type) * rand_velocity_mul.0,
                    false => velocity.0.x = -unit_speed(&unit.unit_type) * rand_velocity_mul.0,
                }
            }
        }
    }
}
