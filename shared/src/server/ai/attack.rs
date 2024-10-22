use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_renet::renet::RenetServer;

use super::UnitBehaviour;
use crate::networking::MultiplayerRoles;
use crate::server::networking::ServerLobby;
use crate::server::physics::movement::Velocity;
use crate::GameState;
use crate::{
    map::GameSceneId,
    networking::{
        Owner, ProjectileType, ServerChannel, ServerMessages, SpawnProjectile, Unit, UnitType,
    },
    BoxCollider, GRAVITY_G,
};

pub struct AttackPlugin;

#[derive(Event)]
pub struct TakeDamage {
    pub target_entity: Entity,
    pub damage: f32,
}

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamage>();

        app.add_systems(
            FixedUpdate,
            (process_attacks, apply_damage).run_if(
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
    mut server: ResMut<RenetServer>,
    mut query: Query<(&UnitBehaviour, &mut Unit, &Owner, &Transform)>,
    position: Query<&Transform>,
    scene_ids: Query<&GameSceneId>,
    mut attack_events: EventWriter<TakeDamage>,
    lobby: Res<ServerLobby>,
    time: Res<Time>,
) {
    for (behaviour, mut unit, owner, transform) in query.iter_mut() {
        if let UnitBehaviour::AttackTarget(target_entity) = behaviour {
            unit.swing_timer.tick(time.delta());
            if unit.swing_timer.finished() {
                match unit.unit_type {
                    UnitType::Shieldwarrior | UnitType::Pikeman => {
                        println!("Swinging at target: {}", target_entity);
                        attack_events.send(TakeDamage {
                            target_entity: *target_entity,
                            damage: unit_damage(&unit.unit_type),
                        });
                    }
                    UnitType::Archer => {
                        let arrow_transform = Transform::from_translation(
                            transform.translation + Vec3::new(0., 1., 0.),
                        );
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
                        let scene_id = scene_ids.get(*target_entity).unwrap();

                        let arrow = commands.spawn((
                            arrow_transform,
                            *owner,
                            projectile_type,
                            velocity,
                            BoxCollider(Vec2::new(20., 20.)),
                            *scene_id,
                        ));

                        let message = ServerMessages::SpawnProjectile(SpawnProjectile {
                            entity: arrow.id(),
                            projectile_type,
                            translation: arrow_transform.translation.into(),
                            direction: velocity.0.into(),
                        });
                        let message = bincode::serialize(&message).unwrap();
                        for (client_id, entity) in lobby.players.iter() {
                            let player_scene_id = scene_ids.get(*entity).unwrap();
                            if scene_id.eq(player_scene_id) {
                                server.send_message(
                                    *client_id,
                                    ServerChannel::ServerMessages,
                                    message.clone(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn apply_damage(mut query: Query<&mut Unit>, mut attack_events: EventReader<TakeDamage>) {
    for event in attack_events.read() {
        if let Ok(mut unit) = query.get_mut(event.target_entity) {
            unit.health -= event.damage;
            println!("New health: {}.", unit.health);
        }
    }
}
