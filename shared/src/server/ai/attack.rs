use bevy::prelude::*;

use bevy_behave::prelude::BehaveCtx;
use bevy_replicon::prelude::{SendMode, ToClients};
use std::f32::consts::FRAC_PI_4;

use super::{AttackingInRange, Target};
use crate::{
    AnimationChange, AnimationChangeEvent, GRAVITY_G, Hitby, Owner,
    map::Layers,
    networking::{UnitType, WorldDirection},
    server::{
        ai::AllowToAttack,
        buildings::recruiting::FlagAssignment,
        entities::{
            Damage, Unit,
            commander::ArmyFlagAssignments,
            health::{DelayedDamage, Health, TakeDamage},
        },
        game_scenes::GameSceneId,
        physics::{movement::Velocity, projectile::ProjectileType},
    },
};

pub struct AIAttackPlugin;

impl Plugin for AIAttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (process_attacks, check_allow_to_attack));
    }
}

fn check_allow_to_attack(
    query: Query<(&BehaveCtx, &AllowToAttack)>,
    mut commands: Commands,
    others: Query<(Entity, &Unit, &Transform), With<Health>>,
    units: Query<(&Transform, &FlagAssignment), With<Health>>,
    army: Query<&ArmyFlagAssignments>,
) {
    for (ctx, allow_to_attack) in query.iter() {
        let unit_entity = ctx.target_entity();
        let (entity, unit, commander_position) = others.get(unit_entity).unwrap();

        match unit.unit_type {
            UnitType::Shieldwarrior | UnitType::Pikeman | UnitType::Archer | UnitType::Bandit => {
                commands.trigger(ctx.success());
            }
            UnitType::Commander => {
                if let Ok(army) = army.get(entity) {
                    let mut all_units_position: Vec<f32> = Vec::new();
                    let commander_position = commander_position.translation.x;

                    for formation_flag in army.flags.iter().flatten() {
                        for (unit_position, flag) in units.iter() {
                            if flag.0 == *formation_flag {
                                all_units_position.push(unit_position.translation.x);
                            }
                        }
                    }

                    let behind_all_units =
                        all_units_position
                            .iter()
                            .all(|&unit_position| match **allow_to_attack {
                                WorldDirection::Left => unit_position < commander_position,
                                WorldDirection::Right => unit_position > commander_position,
                            });

                    if behind_all_units {
                        commands.trigger(ctx.success());
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn process_attacks(
    query: Query<&BehaveCtx, With<AttackingInRange>>,
    mut commands: Commands,
    mut unit: Query<(
        &mut Unit,
        Option<&Target>,
        &Owner,
        &Transform,
        &Damage,
        &GameSceneId,
    )>,
    mut animation: EventWriter<ToClients<AnimationChangeEvent>>,
    position: Query<&Transform>,
) {
    for ctx in query.iter() {
        let entity = ctx.target_entity();
        let Ok((mut unit, maybe_target, owner, transform, damage, game_scene_id)) =
            unit.get_mut(entity)
        else {
            continue;
        };
        let Some(target) = maybe_target else {
            commands.trigger(ctx.success());
            continue;
        };

        if unit.swing_timer.finished() {
            let target_pos = if let Ok(target_transform) = position.get(**target) {
                target_transform.translation
            } else {
                commands.trigger(ctx.success());
                continue;
            };
            let delta_x = target_pos.x - transform.translation.x;

            match unit.unit_type {
                UnitType::Shieldwarrior
                | UnitType::Pikeman
                | UnitType::Bandit
                | UnitType::Commander => {
                    commands.spawn(DelayedDamage::new(
                        &unit.unit_type,
                        TakeDamage {
                            target_entity: **target,
                            damage: **damage,
                            direction: delta_x.into(),
                            by: Hitby::Melee,
                        },
                    ));
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
                event: AnimationChangeEvent {
                    entity,
                    change: AnimationChange::Attack,
                },
            });

            unit.swing_timer.reset();
        }
    }
}
