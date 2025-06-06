use avian2d::{
    math::{Scalar, Vector},
    prelude::{
        ColliderOf, Collisions, LinearVelocity, NarrowPhaseSet, PhysicsLayer, PhysicsSchedule,
        Position, RigidBody, Sensor,
    },
};
use bevy::prelude::*;

use crate::{
    Owner, Player,
    server::{
        ai::{FollowOffset, UnitBehaviour},
        entities::{Unit, health::Health},
        physics::{
            PushBack,
            movement::{Grounded, RandomVelocityMul, Speed, Velocity},
            projectile::ProjectileType,
        },
    },
};

#[derive(PhysicsLayer, Default)]
pub enum GameLayer {
    #[default]
    Player,
    Wall,
    Ground,
}

pub struct AvianPlugin;

impl Plugin for AvianPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                set_grounded,
                set_unit_velocity,
                apply_velocity,
                apply_gravity,
                set_projectile_rotation,
                apply_movement_damping,
            )
                .chain(),
        );
        app.add_systems(
            PhysicsSchedule,
            kinematic_controller_collisions.in_set(NarrowPhaseSet::Last),
        );
    }
}

fn apply_velocity(
    mut query: Query<(&LinearVelocity, &mut Transform), Changed<Velocity>>,
    time: Res<Time>,
) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.) * time.delta_secs();
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

fn set_grounded(mut commands: Commands, entities: Query<(Entity, &Transform)>) {
    for (entity, transform) in &entities {
        let Ok(mut entity) = commands.get_entity(entity) else {
            continue;
        };

        if transform.translation.y == 0. {
            entity.try_insert(Grounded);
        } else {
            entity.try_remove::<Grounded>();
        }
    }
}

#[allow(clippy::type_complexity)]
fn set_unit_velocity(
    mut query: Query<
        (
            &mut Velocity,
            &mut Transform,
            &UnitBehaviour,
            &FollowOffset,
            &PushBack,
            &RandomVelocityMul,
            &Speed,
        ),
        (With<Unit>, With<Health>),
    >,
    transform_query: Query<&Transform, Without<Unit>>,
) {
    for (
        mut velocity,
        mut transform,
        behaviour,
        follow_offset,
        push_back,
        rand_velocity_mul,
        speed,
    ) in &mut query
    {
        match behaviour {
            UnitBehaviour::Idle => {}
            UnitBehaviour::AttackTarget(_) => {
                if !push_back.timer.finished() {
                    continue;
                }
                velocity.0.x = 0.;
            }
            UnitBehaviour::FollowFlag(flag) => {
                let target = transform_query.get(*flag).unwrap().translation.truncate();

                let target = target + **follow_offset;
                let direction = (target.x - transform.translation.x).signum();

                if (transform.translation.x - target.x).abs() <= 1. {
                    velocity.0.x = 0.;
                    continue;
                }

                velocity.0.x = direction * **speed * rand_velocity_mul.0;
                transform.scale.x = direction;
            }
        }
    }
}

fn apply_gravity(time: Res<Time>, mut controllers: Query<(&mut LinearVelocity)>) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs();
    for (mut linear_velocity) in &mut controllers {
        linear_velocity.0 += (Vector::NEG_Y * 9.81 * 2.0) * delta_time;
    }
}

fn apply_movement_damping(mut query: Query<(&mut LinearVelocity)>) {
    for mut linear_velocity in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= 0.52;
    }
}

fn kinematic_controller_collisions(
    collisions: Collisions,
    bodies: Query<&RigidBody>,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    owners: Query<&Owner>,
    mut character_controllers: Query<(&mut Position, &mut LinearVelocity), With<Player>>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        let Ok([&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }]) =
            collider_rbs.get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        let object: Entity;
        // Get the body of the character controller and whether it is the first
        // or second entity in the collision.
        let is_first: bool;

        let is_other_dynamic: bool;

        let ((mut position, mut linear_velocity), player_entity) =
            if let Ok(character) = character_controllers.get_mut(rb1) {
                is_first = true;
                is_other_dynamic = bodies.get(rb2).is_ok_and(|rb| rb.is_dynamic());
                object = rb2;
                (character, rb1)
            } else if let Ok(character) = character_controllers.get_mut(rb2) {
                is_first = false;
                is_other_dynamic = bodies.get(rb1).is_ok_and(|rb| rb.is_dynamic());
                object = rb1;
                (character, rb2)
            } else {
                continue;
            };

        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.normal
            } else {
                manifold.normal
            };

            let mut deepest_penetration: Scalar = Scalar::MIN;

            // Solve each penetrating contact in the manifold.
            for contact in manifold.points.iter() {
                if contact.penetration > 0.0 {
                    position.0 += normal * contact.penetration;
                }
                deepest_penetration = deepest_penetration.max(contact.penetration);
            }

            // For now, this system only handles velocity corrections for collisions against static geometry.
            if is_other_dynamic {
                continue;
            }

            info!(deepest_penetration);

            if deepest_penetration > 0.0 {
                // The character is intersecting an unclimbable object, like a wall.
                info!("wall");
                // Don't apply an impulse if the character is moving away from the surface.
                if linear_velocity.dot(normal) > 0.0 {
                    continue;
                }

                if let Ok(object_owner) = owners.get(object) {
                    if object_owner.is_same_faction(&Owner::Player(player_entity)) {
                        continue;
                    }
                };

                // Slide along the surface, rejecting the velocity along the contact normal.
                let impulse = linear_velocity.reject_from_normalized(normal);
                linear_velocity.0 = impulse;
            } else {
                linear_velocity.0.x = 0.;
            }
        }
    }
}
