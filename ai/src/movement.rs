use bevy::prelude::*;

use army::flag::FlagAssignment;
use bevy::math::bounding::IntersectsVolume;
use bevy_behave::prelude::{BehaveCtx, BehaveTrigger};
use physics::{
    attachment::AttachedTo,
    movement::{BoxCollider, RandomVelocityMul, Speed, Velocity},
};
use shared::{GameScene, GameSceneId};
use travel::Traveling;
use units::{MeleeRange, ProjectileRange, Unit, UnitType};

use crate::{ArmyFormationTo, ArmyFormations, TravelToEntity, WalkIntoRange};

use super::{Attack, FormationHasTarget, Target, offset::FollowOffset};

#[derive(Component, Clone)]
pub struct FollowFlag;

#[derive(Component, Clone)]
pub struct IsFriendlyFormationUnitInFront;

#[derive(Component, Clone)]
pub struct IsFriendlyUnitInFront;
#[derive(Component, Clone, Deref, DerefMut)]
pub struct Roam(Timer);

impl Default for Roam {
    fn default() -> Self {
        Self(Timer::from_seconds(
            fastrand::f32() * 2. + 1.,
            TimerMode::Repeating,
        ))
    }
}

pub struct AIMovementPlugin;

impl Plugin for AIMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                follow_flag,
                roam,
                walk_into_range,
                walk_in_direction,
                travel_to_entity,
            ),
        );
        app.add_observer(friendly_formation_unit_in_front);
        app.add_observer(friendly_unit_in_front);
        app.add_observer(formation_has_target);
    }
}

const MOVE_EPSILON: f32 = 1.;

fn follow_flag(
    query: Query<&BehaveCtx, With<FollowFlag>>,
    mut unit: Query<(
        &mut Velocity,
        &Transform,
        &FollowOffset,
        &RandomVelocityMul,
        &Speed,
        &FlagAssignment,
        &Unit,
        Option<&Target>,
    )>,
    is_attached: Query<&AttachedTo>,
    transform_query: Query<&Transform>,
) -> Result {
    for ctx in query.iter() {
        let (
            mut velocity,
            transform,
            offset,
            rand_velocity_mul,
            speed,
            flag_assignment,
            unit,
            has_taget,
        ) = unit.get_mut(ctx.target_entity())?;

        if has_taget.is_some() {
            return Ok(());
        }

        let flag_pos = transform_query
            .get(**flag_assignment)
            .unwrap()
            .translation
            .truncate();
        let attached_to = is_attached.get(**flag_assignment)?;
        let building_pos = transform_query.get(**attached_to)?.translation.truncate();

        let desired = building_pos + **offset;

        let diff = desired - transform.translation.truncate();

        if diff.length() > MOVE_EPSILON {
            velocity.0 = diff.normalize() * **speed * **rand_velocity_mul;
        } else {
            velocity.0 = Vec2::ZERO;
        }

        if unit.unit_type.eq(&UnitType::Commander) {
            velocity.0.x = if (flag_pos.x - transform.translation.x).abs() > MOVE_EPSILON {
                (flag_pos.x - transform.translation.x).signum() * **speed
            } else {
                0.
            };
        }
    }

    Ok(())
}

fn roam(
    query: Query<&BehaveCtx, With<Roam>>,
    mut unit: Query<(&mut Roam, &mut Velocity, &RandomVelocityMul, &Speed)>,
    time: Res<Time>,
) -> Result {
    for ctx in query.iter() {
        let (mut roam, mut velocity, rand_velocity_mul, speed) =
            unit.get_mut(ctx.target_entity())?;
        roam.tick(time.delta());

        if roam.just_finished() {
            let x = (fastrand::f32() - 0.5) * 2.;
            velocity.0.x = x.signum() * **speed * **rand_velocity_mul;
        }
    }
    Ok(())
}

fn walk_into_range(
    query: Query<&BehaveCtx, With<WalkIntoRange>>,
    mut unit: Query<(
        &mut Velocity,
        &Transform,
        Option<&Target>,
        &RandomVelocityMul,
        &Speed,
        Option<&ProjectileRange>,
        Option<&MeleeRange>,
    )>,
    transform_query: Query<&Transform>,
    mut commands: Commands,
) -> Result {
    for ctx in query.iter() {
        let (
            mut velocity,
            transform,
            maybe_target,
            rand_velocity_mul,
            speed,
            projectile_range,
            melee_range,
        ) = unit.get_mut(ctx.target_entity())?;

        let Some(target) = maybe_target else {
            commands.trigger(ctx.failure());
            continue;
        };

        let target = transform_query.get(**target)?.translation.truncate();

        let range = if let Some(r) = projectile_range {
            **r
        } else if let Some(r) = melee_range {
            **r
        } else {
            commands.trigger(ctx.failure());
            continue;
        };

        let direction = (target.x - transform.translation.x).signum();
        if (transform.translation.x - target.x).abs() <= range {
            velocity.0.x = 0.;
            commands.trigger(ctx.success());
            continue;
        }

        velocity.0.x = direction * **speed * **rand_velocity_mul;
    }

    Ok(())
}

fn friendly_formation_unit_in_front(
    trigger: On<BehaveTrigger<IsFriendlyFormationUnitInFront>>,
    units: Query<(&Unit, Option<&ArmyFormationTo>)>,
    other_units: Query<(Entity, &Transform, &Unit, &BoxCollider)>,
    commander: Query<&ArmyFormations>,
    mut commands: Commands,
) -> Result {
    let target = trigger.ctx().target_entity();
    let ctx = trigger.event().ctx();

    // Get army formation for target
    let (_unit, army_formation) = units.get(target)?;

    let army_formation = match army_formation {
        Some(formation) => formation,
        None => {
            commands.trigger(ctx.failure());
            return Ok(());
        }
    };

    // Get all formation units
    let entities = commander.get(**army_formation)?;
    let position = entities.iter().position(|entity| entity == target);

    match position {
        Some(pos) => {
            if pos == 0 {
                commands.trigger(ctx.failure());
                return Ok(());
            } else {
                // Get the unit in front (previous position in formation)
                let unit_in_front = entities.iter().nth(pos - 1).unwrap();

                // Get target and front unit data
                if let (Ok((_, transform_1, _, box_1)), Ok((_, transform_2, _, box_2))) =
                    (other_units.get(target), other_units.get(unit_in_front))
                {
                    // Check for collision using Bevy's intersects method
                    if box_1.at(transform_1).intersects(&box_2.at(transform_2)) {
                        commands.trigger(ctx.success());
                    } else {
                        commands.trigger(ctx.failure());
                    }
                } else {
                    // Unit in front not found or doesn't have required components
                    commands.trigger(ctx.failure());
                }
            }
        }
        None => {
            commands.trigger(ctx.failure());
            return Ok(());
        }
    }
    Ok(())
}

fn friendly_unit_in_front(
    trigger: On<BehaveTrigger<IsFriendlyUnitInFront>>,
    units: Query<(&Unit, &Transform)>,
    other_units: Query<(Entity, &Transform, &Unit)>,
    mut commands: Commands,
) -> Result {
    let target = trigger.ctx().target_entity();
    let ctx = trigger.event().ctx();

    // Get army formation for target
    let (target_unit, target_transform) = units.get(target)?;
    if target_unit.unit_type.eq(&UnitType::Shieldwarrior) {
        commands.trigger(ctx.failure());
        return Ok(());
    }

    let target_pos = target_transform.translation.truncate();
    let target_direction = target_transform.forward().truncate();

    let is_unit_in_front = other_units
        .iter()
        .filter(|(entity, _, _)| *entity != target)
        .any(|(_, other_transform, _)| {
            let other_pos = other_transform.translation.truncate();
            let to_other = (other_pos - target_pos).normalize();
            let distance = target_pos.distance(other_pos);

            distance < 20. && to_other.dot(target_direction) > 0.5
        });

    if is_unit_in_front {
        commands.trigger(ctx.success());
    } else {
        commands.trigger(ctx.failure());
    }

    Ok(())
}

fn formation_has_target(
    trigger: On<BehaveTrigger<FormationHasTarget>>,
    has_target: Query<&Target>,
    formation_to_query: Query<&ArmyFormationTo>,
    commander: Query<&ArmyFormations>,
    mut commands: Commands,
) -> Result {
    let ctx = trigger.event().ctx();
    let target_entity = ctx.target_entity();

    if has_target.get(target_entity).is_ok() {
        commands.trigger(ctx.failure());
        return Ok(());
    }

    let formation_to = formation_to_query.get(target_entity)?;

    let formation = commander.get(**formation_to)?;

    let any_in_formation_has_target = formation
        .iter()
        .any(|entity| has_target.get(entity).is_ok());

    if any_in_formation_has_target {
        commands.trigger(ctx.success());
    } else {
        commands.trigger(ctx.failure());
    }
    Ok(())
}

fn walk_in_direction(
    query: Query<&BehaveCtx, With<Attack>>,
    mut unit: Query<(
        &mut Velocity,
        &RandomVelocityMul,
        &Speed,
        Option<&ArmyFormationTo>,
    )>,
    has_target: Query<Option<&Target>>,
    commander: Query<&ArmyFormations>,
    target_location: Query<&Transform>,
) -> Result {
    for ctx in query.iter() {
        let (_, _, _, has_formation) = unit.get_mut(ctx.target_entity())?;

        let army_formation = match has_formation {
            Some(formation) => formation,
            None => {
                return Ok(());
            }
        };

        let formation = commander.get(**army_formation)?;

        let any_has_target = formation
            .iter()
            .find(|entity| has_target.get(*entity).is_ok());

        let unit_transform = target_location.get(ctx.target_entity())?;

        if let Some(target) = any_has_target {
            let target_transform = target_location.get(target)?;
            let direction = (target_transform.translation - unit_transform.translation)
                .signum()
                .x;

            if let Ok((mut entity_velocity, rand_velocity_mul, speed, _)) =
                unit.get_mut(ctx.target_entity())
            {
                entity_velocity.0.x = direction * **speed * **rand_velocity_mul;
            }
        }
    }
    Ok(())
}

fn travel_to_entity(
    query: Query<(&BehaveCtx, &TravelToEntity)>,
    mut traveler: Query<(&mut Velocity, &Speed, Option<&Traveling>)>,
    world_position: Query<(&Transform, &GameSceneId)>,
    game_scenes: Query<&GameScene>,
    mut commands: Commands,
) -> Result {
    for (ctx, travel_target) in query.iter() {
        let entity = ctx.target_entity();
        let (mut velocity, speed, maybe_traveling) = traveler.get_mut(entity)?;

        if maybe_traveling.is_some() {
            continue;
        };

        let (traveler_transform, traveler_scene_id) = world_position.get(entity)?;
        let (target_transform, target_scene_id) = world_position.get(**travel_target)?;

        if traveler_scene_id == target_scene_id {
            let diff = target_transform.translation - traveler_transform.translation;
            if diff.length() > 10. {
                velocity.0 = diff.truncate().normalize() * **speed;
            } else {
                velocity.0 = Vec2::ZERO;
                commands.trigger(ctx.success());
            }
        } else {
            let source_game_scene = game_scenes
                .iter()
                .find(|scene| scene.id == *traveler_scene_id);
            let target_game_scene = game_scenes
                .iter()
                .find(|scene| scene.id == *target_scene_id);

            if let (Some(source), Some(target)) = (source_game_scene, target_game_scene) {
                commands
                    .entity(entity)
                    .insert(Traveling::between(*source, *target))
                    .remove::<GameSceneId>();
                velocity.0 = Vec2::ZERO;
            } else {
                commands.trigger(ctx.failure());
            }
        }
    }
    Ok(())
}
