use bevy::prelude::*;

use crate::{
    map::{buildings::Building, GameSceneId},
    networking::{MultiplayerRoles, Owner, PlayerInput},
    server::{
        ai::{attack::unit_speed, UnitBehaviour},
        entities::{health::Health, Unit},
    },
    BoxCollider, GameState, GRAVITY_G,
};
use bevy::math::bounding::{Aabb2d, IntersectsVolume};

use super::PushBack;

#[derive(Debug, Default, Component, Copy, Clone)]
pub struct Velocity(pub Vec2);

const PLAYER_MOVE_SPEED: f32 = 200.0;

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
            )
                .run_if(in_state(GameState::GameSession).and(in_state(MultiplayerRoles::Host))),
        );
    }
}

fn apply_gravity(mut query: Query<(&mut Velocity, &Transform, &BoxCollider)>, time: Res<Time>) {
    for (mut velocity, transform, collider) in &mut query {
        let bottom = transform.translation.y - collider.half_size().y;
        let next_bottom = bottom + velocity.0.y * time.delta_secs();

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
        &PlayerInput,
        &mut Velocity,
        &Transform,
        &BoxCollider,
        &GameSceneId,
        &Owner,
    )>,
    buildings: Query<(&Transform, &BoxCollider, &GameSceneId, &Owner, &Building), With<Health>>,
) {
    for (input, mut velocity, player_transform, player_collider, player_scene, client_owner) in
        query.iter_mut()
    {
        let x = (input.right as i8 - input.left as i8) as f32;
        let direction = Vec2::new(x, 0.).normalize_or_zero();
        let desired_velocity = direction * PLAYER_MOVE_SPEED;

        let future_position = player_transform.translation.truncate() + direction;
        let future_bounds = Aabb2d::new(future_position, player_collider.half_size());

        let mut would_collide = false;
        for (building_transform, building_collider, builing_scene, owner, building) in
            buildings.iter()
        {
            if player_scene.ne(builing_scene) {
                continue;
            }

            if owner.0.eq(&client_owner.0) {
                continue;
            }

            if Building::Wall.ne(building) {
                continue;
            }

            let building_bounds = Aabb2d::new(
                building_transform.translation.truncate(),
                building_collider.half_size(),
            );

            if building_bounds.intersects(&future_bounds) {
                would_collide = true;
                break;
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

                if (transform.translation.x - target.x).abs() <= MOVE_EPSILON {
                    velocity.0.x = 0.;
                    continue;
                }

                let target_right = target.x > transform.translation.x;

                match target_right {
                    true => velocity.0.x = unit_speed(&unit.unit_type),
                    false => velocity.0.x = -unit_speed(&unit.unit_type),
                }
            }
        }
    }
}
