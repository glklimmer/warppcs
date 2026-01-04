use bevy::prelude::*;

use army::{ArmyFlagAssignments, flag::FlagAssignment};
use bevy_behave::prelude::BehaveCtx;
use bevy_replicon::prelude::{SendMode, ToClients};
use health::{DelayedDamage, Health, TakeDamage};
use physics::{movement::Velocity, projectile::ProjectileType};
use shared::{
    AnimationChange, AnimationChangeEvent, GRAVITY_G, GameSceneId, Hitby, Owner, map::Layers,
    networking::WorldDirection,
};
use std::f32::consts::FRAC_PI_4;
use units::{Damage, Unit, UnitType};

use super::{Attack, Target, WaitToAttack};

pub(crate) struct AIAttackPlugin;

impl Plugin for AIAttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (process_attacks, wait_to_attack));
    }
}

fn wait_to_attack(
    query: Query<(&BehaveCtx, &WaitToAttack)>,
    mut commands: Commands,
    others: Query<(Entity, &Unit, &Transform), With<Health>>,
    units: Query<(&Transform, &FlagAssignment), With<Health>>,
    army: Query<&ArmyFlagAssignments>,
) -> Result {
    for (ctx, allow_to_attack) in query.iter() {
        let unit_entity = ctx.target_entity();
        let (entity, unit, commander_position) = others.get(unit_entity)?;

        match unit.unit_type {
            UnitType::Shieldwarrior | UnitType::Pikeman | UnitType::Archer | UnitType::Bandit => {
                commands.trigger(ctx.success());
            }
            UnitType::Commander => {
                let army = army.get(entity)?;

                let comparer: fn(f32, f32) -> bool = match **allow_to_attack {
                    WorldDirection::Left => |a, b| a > b,
                    WorldDirection::Right => |a, b| a < b,
                };

                let commander_position = commander_position.translation.x;
                let mut extreme = commander_position;

                for formation_flag in army.flags.iter().flatten() {
                    for (unit_position, flag) in units.iter() {
                        if **flag == *formation_flag
                            && comparer(unit_position.translation.x, extreme)
                        {
                            extreme = unit_position.translation.x;
                        }
                    }
                }

                if (extreme - commander_position).abs() <= 0. {
                    commands.trigger(ctx.success());
                }
            }
        }
    }
    Ok(())
}

fn process_attacks(
    query: Query<(&BehaveCtx, &Attack)>,
    mut commands: Commands,
    mut unit: Query<(
        &mut Unit,
        &Target,
        &Owner,
        &Transform,
        &Damage,
        &GameSceneId,
    )>,
    mut animation: MessageWriter<ToClients<AnimationChangeEvent>>,
    position: Query<&Transform>,
) {
    for (ctx, attacking_range) in query.iter() {
        let entity = ctx.target_entity();
        let Ok((mut unit, target, owner, transform, damage, game_scene_id)) = unit.get_mut(entity)
        else {
            commands.trigger(ctx.failure());
            continue;
        };

        if !unit.swing_timer.is_finished() {
            continue;
        }

        let target_pos = if let Ok(target_transform) = position.get(**target) {
            target_transform.translation
        } else {
            commands.trigger(ctx.failure());
            continue;
        };
        let delta_x = target_pos.x - transform.translation.x;

        match attacking_range {
            Attack::Melee => {
                commands.spawn(&unit.unit_type.attack_delayed(TakeDamage {
                    target_entity: **target,
                    damage: **damage,
                    direction: delta_x.into(),
                    by: Hitby::Melee,
                }));
            }
            Attack::Projectile => {
                let arrow_position = Vec3::new(
                    transform.translation.x,
                    transform.translation.y + 5.,
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

                commands.spawn((
                    arrow_transform,
                    *owner,
                    projectile_type,
                    velocity,
                    Damage(**damage),
                    *game_scene_id,
                ));
            }
        }

        animation.write(ToClients {
            mode: SendMode::Broadcast,
            message: AnimationChangeEvent {
                entity,
                change: AnimationChange::Attack,
            },
        });
        unit.swing_timer.reset();
    }
}
