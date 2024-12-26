use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;

use super::UnitBehaviour;
use crate::{
    map::{GameSceneId, Layers},
    networking::{
        MultiplayerRoles, Owner, ProjectileType, ServerMessages, SpawnProjectile, UnitType,
    },
    server::{
        entities::{health::TakeDamage, Unit},
        networking::SendServerMessage,
        physics::movement::Velocity,
    },
    BoxCollider, GameState, GRAVITY_G,
};

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (process_attacks).run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
    }
}

pub fn unit_range(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 50.,
        UnitType::Pikeman => 140.,
        UnitType::Archer => 600.,
    }
}

pub fn unit_damage(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 12.,
        UnitType::Pikeman => 18.,
        UnitType::Archer => 6.,
    }
}

pub fn unit_health(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 120.,
        UnitType::Pikeman => 90.,
        UnitType::Archer => 60.,
    }
}

pub fn unit_swing_timer(unit_type: &UnitType) -> Timer {
    let time = match unit_type {
        UnitType::Shieldwarrior => 1.,
        UnitType::Pikeman => 2.,
        UnitType::Archer => 4.,
    };
    Timer::from_seconds(time, TimerMode::Repeating)
}

pub fn unit_speed(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 80.,
        UnitType::Pikeman => 100.,
        UnitType::Archer => 120.,
    }
}

pub fn projectile_damage(projectile_type: &ProjectileType) -> f32 {
    match projectile_type {
        ProjectileType::Arrow => 15.,
    }
}

#[allow(clippy::too_many_arguments)]
fn process_attacks(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &UnitBehaviour,
        &mut Unit,
        &Owner,
        &Transform,
        &GameSceneId,
    )>,
    mut attack_events: EventWriter<TakeDamage>,
    mut sender: EventWriter<SendServerMessage>,
    position: Query<&Transform>,
    scene_ids: Query<&GameSceneId>,
    time: Res<Time>,
) {
    for (entity, behaviour, mut unit, owner, transform, game_scene_id) in query.iter_mut() {
        if let UnitBehaviour::AttackTarget(target_entity) = behaviour {
            unit.swing_timer.tick(time.delta());
            if unit.swing_timer.finished() {
                let target_scene_id = scene_ids.get(*target_entity).unwrap();

                match unit.unit_type {
                    UnitType::Shieldwarrior | UnitType::Pikeman => {
                        println!("Swinging at target: {}", target_entity);
                        attack_events.send(TakeDamage {
                            target_entity: *target_entity,
                            damage: unit_damage(&unit.unit_type),
                        });
                        sender.send(SendServerMessage {
                            message: ServerMessages::MeleeAttack { entity },
                            game_scene_id: *game_scene_id,
                        });
                    }
                    UnitType::Archer => {
                        let arrow_position = Vec3::new(
                            transform.translation.x,
                            transform.translation.y + 1.,
                            Layers::Projectile.as_f32(),
                        );
                        let arrow_transform = Transform::from_translation(arrow_position);
                        let projectile_type = ProjectileType::Arrow;

                        let target_pos = if let Ok(target_transform) = position.get(*target_entity)
                        {
                            target_transform.translation
                        } else {
                            continue;
                        };
                        let delta_x = target_pos.x - transform.translation.x;
                        let delta_y = target_pos.y - transform.translation.y;

                        let theta = if delta_x > 0. { FRAC_PI_4 } else { 2.3561944 };

                        let cos_theta = theta.cos();
                        let sin_theta = theta.sin();
                        let tan_theta = sin_theta / cos_theta;
                        let cos_theta_squared = cos_theta * cos_theta;

                        let denominator = 2.0 * (delta_x * tan_theta - delta_y) * cos_theta_squared;
                        if denominator <= 0.0 {
                            println!(
                                "Shooting not possible, theta: {}, delta_x: {}",
                                theta, delta_x
                            );
                            continue;
                        }

                        let numerator = GRAVITY_G * delta_x * delta_x;

                        let v0_squared = numerator / denominator;
                        let speed = v0_squared.sqrt();

                        let velocity = Velocity(Vec2::from_angle(theta) * speed);

                        let arrow = commands.spawn((
                            arrow_transform,
                            *owner,
                            projectile_type,
                            velocity,
                            BoxCollider(Vec2::new(20., 20.)),
                            *target_scene_id,
                        ));
                        println!("arrow spawn: {:?}", target_scene_id);

                        sender.send(SendServerMessage {
                            message: ServerMessages::SpawnProjectile(SpawnProjectile {
                                entity: arrow.id(),
                                projectile_type,
                                translation: arrow_transform.translation.into(),
                                direction: velocity.0.into(),
                            }),
                            game_scene_id: *target_scene_id,
                        });
                    }
                }
            }
        }
    }
}
