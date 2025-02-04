use bevy::prelude::*;

use crate::{
    map::{
        buildings::{BuildStatus, Building},
        GameSceneId,
    },
    networking::{Owner, PlayerInput},
    server::{
        ai::{attack::unit_speed, UnitBehaviour},
        entities::{health::Health, Unit},
    },
    BoxCollider, GRAVITY_G,
};
use bevy::math::bounding::IntersectsVolume;

use super::PushBack;

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct Velocity(pub Vec2);

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
const COLLISION_EPSILON: f32 = 5.;

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
    buildings: Query<
        (
            &Transform,
            &BoxCollider,
            &GameSceneId,
            &Owner,
            &Building,
            &BuildStatus,
        ),
        With<Health>,
    >,
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

        let future_position =
            player_transform.translation.truncate() + direction + COLLISION_EPSILON;
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

            if building_status.ne(&BuildStatus::Built) {
                continue;
            }

            if let Building::Wall { level: _ } = building {
                let building_bounds = building_collider.at(building_transform);

                if building_bounds.intersects(&future_bounds) {
                    println!("collisoin");
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
        &BoxCollider,
        &UnitBehaviour,
        &Unit,
        &PushBack,
        &Owner,
        &GameSceneId,
    )>,
    transform_query: Query<&Transform>,
    buildings: Query<
        (
            &Transform,
            &BoxCollider,
            &GameSceneId,
            &Owner,
            &Building,
            &BuildStatus,
        ),
        With<Health>,
    >,
) {
    for (mut velocity, transform, collider, behaviour, unit, push_back, client_owner, unit_scene) in
        &mut query
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
                let target_right = target.x > transform.translation.x;

                let future_position = transform.translation.truncate()
                    * (if target_right { 1. } else { -1. })
                    + COLLISION_EPSILON;
                let future_bounds = collider.at_pos(future_position);

                let mut would_collide = false;
                for (
                    building_transform,
                    building_collider,
                    builing_scene,
                    building_owner,
                    building,
                    building_status,
                ) in buildings.iter()
                {
                    if unit_scene.ne(builing_scene) {
                        continue;
                    }

                    if client_owner.eq(building_owner) {
                        continue;
                    }

                    if building_status.ne(&BuildStatus::Built) {
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

                if (transform.translation.x - target.x).abs() <= MOVE_EPSILON || would_collide {
                    velocity.0.x = 0.;
                    continue;
                }

                match target_right {
                    true => velocity.0.x = unit_speed(&unit.unit_type),
                    false => velocity.0.x = -unit_speed(&unit.unit_type),
                }
            }
        }
    }
}
