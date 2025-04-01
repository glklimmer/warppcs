use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_replicon::prelude::{SendMode, ToClients};

use super::UnitBehaviour;
use crate::{
    map::Layers,
    networking::{Facing, UnitType},
    server::{
        entities::{health::TakeDamage, Unit},
        physics::{movement::Velocity, projectile::ProjectileType},
    },
    AnimationChange, AnimationChangeEvent, Hitby, Owner, GRAVITY_G,
};

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, process_attacks);
    }
}

pub fn unit_range(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 20.,
        UnitType::Pikeman => 30.,
        UnitType::Archer => 200.,
        UnitType::Bandit => 20.,
    }
}

pub fn unit_damage(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 12.,
        UnitType::Pikeman => 18.,
        UnitType::Archer => 6.,
        UnitType::Bandit => 14.,
    }
}

pub fn unit_health(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 120.,
        UnitType::Pikeman => 90.,
        UnitType::Archer => 60.,
        UnitType::Bandit => 100.,
    }
}

pub fn unit_swing_timer(unit_type: &UnitType) -> Timer {
    let time = match unit_type {
        UnitType::Shieldwarrior => 1.,
        UnitType::Pikeman => 2.,
        UnitType::Archer => 4.,
        UnitType::Bandit => 3.,
    };
    Timer::from_seconds(time, TimerMode::Repeating)
}

pub fn unit_speed(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 25.,
        UnitType::Pikeman => 33.,
        UnitType::Archer => 45.,
        UnitType::Bandit => 33.,
    }
}

pub fn projectile_damage(projectile_type: &ProjectileType) -> f32 {
    match projectile_type {
        ProjectileType::Arrow => 15.,
    }
}

fn process_attacks(
    mut commands: Commands,
    mut query: Query<(Entity, &UnitBehaviour, &mut Unit, &Owner, &Transform)>,
    mut attack_events: EventWriter<TakeDamage>,
    mut animation: EventWriter<ToClients<AnimationChangeEvent>>,
    position: Query<&Transform>,
    time: Res<Time>,
) {
    for (entity, behaviour, mut unit, owner, transform) in query.iter_mut() {
        if let UnitBehaviour::AttackTarget(target_entity) = behaviour {
            unit.swing_timer.tick(time.delta());
            if unit.swing_timer.finished() {
                let target_pos = if let Ok(target_transform) = position.get(*target_entity) {
                    target_transform.translation
                } else {
                    continue;
                };
                let delta_x = target_pos.x - transform.translation.x;

                match unit.unit_type {
                    UnitType::Shieldwarrior | UnitType::Pikeman | UnitType::Bandit => {
                        attack_events.send(TakeDamage {
                            target_entity: *target_entity,
                            damage: unit_damage(&unit.unit_type),
                            direction: match delta_x > 0. {
                                true => Facing::Left,
                                false => Facing::Right,
                            },
                            by: Hitby::Meele,
                        });
                        animation.send(ToClients {
                            mode: SendMode::Broadcast,
                            event: AnimationChangeEvent {
                                entity,
                                change: AnimationChange::Attack,
                            },
                        });
                    }
                    UnitType::Archer => {
                        let arrow_position = Vec3::new(
                            transform.translation.x,
                            transform.translation.y + 1.,
                            Layers::Projectile.as_f32(),
                        );

                        let projectile_type = ProjectileType::Arrow;
                        let target_pos = target_pos.with_y(14.);

                        let delta_y = target_pos.y - transform.translation.y;

                        let theta = if delta_x > 0. { FRAC_PI_4 } else { 2.3561944 };

                        let cos_theta = theta.cos();
                        let sin_theta = theta.sin();
                        let tan_theta = sin_theta / cos_theta;
                        let cos_theta_squared = cos_theta * cos_theta;

                        let denominator = 2.0 * (delta_x * tan_theta - delta_y) * cos_theta_squared;
                        if denominator <= 0.0 {
                            warn!(
                                "Shooting not possible, theta: {}, delta_x: {}",
                                theta, delta_x
                            );
                            continue;
                        }

                        let numerator = GRAVITY_G * delta_x * delta_x;

                        let v0_squared = numerator / denominator;
                        let speed = v0_squared.sqrt();

                        let velocity = Velocity(Vec2::from_angle(theta) * speed);

                        let arrow_transform = Transform {
                            translation: arrow_position,
                            scale: Vec3::splat(1.0),
                            rotation: Quat::from_rotation_z(theta),
                        };

                        commands.spawn((arrow_transform, *owner, projectile_type, velocity));

                        animation.send(ToClients {
                            mode: SendMode::Broadcast,
                            event: AnimationChangeEvent {
                                entity,
                                change: AnimationChange::Attack,
                            },
                        });
                    }
                }
            }
        }
    }
}
