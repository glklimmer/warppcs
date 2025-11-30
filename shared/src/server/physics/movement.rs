use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use bevy_replicon::prelude::server_or_singleplayer;
use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, GRAVITY_G, GameSceneId, Owner, Player,
    map::buildings::{BuildStatus, Building, BuildingType},
    networking::WorldDirection,
    server::{entities::health::Health, physics::army_slot::ArmySlot, players::items::Item},
};

use super::projectile::ProjectileType;

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

#[derive(Component, Deref)]
pub struct RandomVelocityMul(f32);

impl Default for RandomVelocityMul {
    fn default() -> Self {
        Self(fastrand::choice([0.9, 0.95, 1.0, 1.1, 1.15]).unwrap())
    }
}

#[derive(Component, Deref)]
#[require(Transform)]
pub struct NoWalkZone(WorldDirection);

impl NoWalkZone {
    pub fn to_the(direction: WorldDirection) -> Self {
        Self(direction)
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                (
                    set_grounded,
                    set_walking,
                    set_king_walking,
                    apply_friction,
                    apply_drag,
                    set_projectile_rotation,
                ),
                (wall_collision, no_walk_zone_collision),
            )
                .chain()
                .run_if(server_or_singleplayer),
        );
        app.add_systems(
            FixedPostUpdate,
            (apply_gravity, (apply_velocity, apply_direction))
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

fn apply_direction(
    mut query: Query<
        (&Velocity, &mut Transform),
        (Changed<Velocity>, Without<Item>, Without<ArmySlot>),
    >,
) {
    for (velocity, mut transform) in query.iter_mut() {
        if velocity.0.x != 0. {
            transform.scale.x = velocity.0.x.signum();
        }
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

fn apply_drag(mut query: Query<&mut Velocity, Without<ProjectileType>>, time: Res<Time>) {
    let drag_coeff = 3.0;
    for mut vel in query.iter_mut() {
        let old = vel.0;
        vel.0 = old - old * drag_coeff * time.delta_secs();
    }
}

fn set_grounded(
    entities: Query<(Entity, &Transform, &BoxCollider)>,
    mut commands: Commands,
) -> Result {
    for (entity, transform, collider) in &entities {
        let mut entity = commands.get_entity(entity)?;

        let bottom = transform.translation.truncate() - collider.half_size()
            + collider.offset.unwrap_or_default();

        if bottom.y <= 0. {
            entity.try_insert(Grounded);
        } else {
            entity.try_remove::<Grounded>();
        }
    }
    Ok(())
}

#[allow(clippy::type_complexity)]
fn set_walking(
    entities: Query<(Entity, &Velocity, Option<&Grounded>, Option<&Health>), Without<Player>>,
    mut commands: Commands,
) -> Result {
    for (entity, velocity, maybe_grounded, maybe_health) in &entities {
        let mut entity = commands.get_entity(entity)?;

        if maybe_health.is_some() && maybe_grounded.is_some() && velocity.0.x.abs() > 0. {
            entity.try_insert(Moving);
        } else {
            entity.try_remove::<Moving>();
        }
    }
    Ok(())
}

fn set_king_walking(
    players: Query<(Entity, &Velocity, Option<&Grounded>), With<Player>>,
    mut commands: Commands,
) -> Result {
    for (entity, velocity, maybe_grounded) in &players {
        let mut entity = commands.get_entity(entity)?;

        if maybe_grounded.is_some() && velocity.0.x.abs() > 0. {
            entity.try_insert(Moving);
        } else {
            entity.try_remove::<Moving>();
        }
    }
    Ok(())
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
            if building_owner.is_same_faction(owner) {
                continue;
            }

            let BuildStatus::Built { indicator: _ } = *building_status else {
                continue;
            };

            if let BuildingType::Wall { level: _ } = building.building_type {
                let building_bounds = building_collider.at(building_transform);

                if building_bounds.intersects(&future_bounds) {
                    velocity.0.x = 0.;
                    break;
                }
            }
        }
    }
}

fn no_walk_zone_collision(
    mut query: Query<(&mut Velocity, &Transform, &GameSceneId), With<BoxCollider>>,
    no_walk_zone: Query<(&Transform, &NoWalkZone, &GameSceneId)>,
    time: Res<Time>,
) {
    for (mut velocity, transform, game_scene_id) in query.iter_mut() {
        let future_position = transform.translation.truncate() + velocity.0 * time.delta_secs();

        for (zone_transform, zone, zone_game_scene_id) in no_walk_zone.iter() {
            if game_scene_id.ne(zone_game_scene_id) {
                continue;
            }

            let overstepped = match **zone {
                WorldDirection::Left => future_position.x <= zone_transform.translation.x,
                WorldDirection::Right => future_position.x >= zone_transform.translation.x,
            };

            if overstepped {
                velocity.0.x = 0.;
                break;
            }
        }
    }
}
