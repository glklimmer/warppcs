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
                (
                    // set_player_velocity
                    set_unit_velocity
                ),
                wall_collision,
            )
                .chain(),
        );
        app.add_systems(FixedPostUpdate, (apply_gravity, apply_velocity).chain());
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

fn wall_collision(
    mut query: Query<(
        &mut Velocity,
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
    time: Res<Time>,
) {
    for (mut velocity, transform, collider, scene, owner) in query.iter_mut() {
        let future_position = transform.translation.truncate() + velocity.0 * time.delta_secs();
        let future_bounds = collider.at_pos(future_position);

        for (
            building_transform,
            building_collider,
            building_scene,
            building_owner,
            building,
            building_status,
        ) in buildings.iter()
        {
            if scene.ne(building_scene) {
                continue;
            }

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

// fn set_player_velocity(mut query: Query<(&mut Velocity, &PlayerInput, &Speed)>) {
//     for (mut velocity, input, speed) in query.iter_mut() {
//         let x = (input.right as i8 - input.left as i8) as f32;
//
//         let direction = Vec2::new(x, 0.).normalize_or_zero();
//         let desired_velocity = direction * speed.0;
//
//         velocity.0 = desired_velocity
//     }
// }

const MOVE_EPSILON: f32 = 1.;

fn set_unit_velocity(
    mut query: Query<(&mut Velocity, &Transform, &UnitBehaviour, &Unit, &PushBack)>,
    transform_query: Query<&Transform>,
) {
    for (mut velocity, transform, behaviour, unit, push_back) in &mut query {
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

                if (transform.translation.x - target.x).abs() <= MOVE_EPSILON {
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
