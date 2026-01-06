use bevy::prelude::*;

use bevy::math::bounding::Aabb2d;
use bevy_replicon::prelude::{AppRuleExt, ClientState};
use serde::{Deserialize, Serialize};
use shared::GameSceneId;

use crate::WorldDirection;

pub const GRAVITY_G: f32 = 9.81 * 33.;

#[derive(Component, Copy, Clone, Default, Deserialize, Serialize)]
pub struct BoxCollider {
    pub dimension: Vec2,
    pub offset: Option<Vec2>,
}

impl BoxCollider {
    pub fn half_size(&self) -> Vec2 {
        Vec2::new(self.dimension.x / 2., self.dimension.y / 2.)
    }

    pub fn at(&self, transform: &Transform) -> Aabb2d {
        Aabb2d::new(
            transform.translation.truncate() + self.offset.unwrap_or_default(),
            self.half_size(),
        )
    }

    pub fn at_pos(&self, position: Vec2) -> Aabb2d {
        Aabb2d::new(position + self.offset.unwrap_or_default(), self.half_size())
    }
}

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

pub(crate) struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<Moving>()
            .replicate::<Grounded>()
            .add_systems(
                FixedUpdate,
                (
                    (
                        set_grounded,
                        set_walking,
                        apply_friction,
                        apply_drag,
                        set_projectile_rotation,
                    ),
                    no_walk_zone_collision,
                )
                    .chain()
                    .run_if(in_state(ClientState::Disconnected)),
            );
        app.add_systems(
            FixedPostUpdate,
            (apply_gravity, (apply_velocity, apply_direction))
                .chain()
                .run_if(in_state(ClientState::Disconnected)),
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

#[derive(Component, Default)]
pub struct Directionless;

fn apply_direction(
    mut query: Query<(&Velocity, &mut Transform), (Changed<Velocity>, Without<Directionless>)>,
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

#[derive(Component, Default)]
pub struct Dragless;

fn apply_drag(mut query: Query<&mut Velocity, Without<Dragless>>, time: Res<Time>) {
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

#[derive(Component)]
pub struct Unmovable;

fn set_walking(
    players: Query<(Entity, &Velocity, Option<&Grounded>), Without<Unmovable>>,
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

#[derive(Component, Default)]
pub struct FreeDirectional;

fn set_projectile_rotation(
    mut projectiles: Query<(&mut Transform, &Velocity), With<FreeDirectional>>,
) {
    for (mut transform, velocity) in projectiles.iter_mut() {
        let angle = velocity.0.to_angle();
        transform.rotation = Quat::from_rotation_z(angle);
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
