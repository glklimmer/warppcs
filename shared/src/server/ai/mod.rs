use bevy::prelude::*;
use bevy_behave::prelude::*;

use attack::AIAttackPlugin;
use bevy_behave::{Behave, behave};
use movement::{AIMovementPlugin, FollowFlag};

use crate::{
    Owner,
    networking::{UnitType, WorldDirection},
    server::{
        ai::{
            bandit::AIBanditPlugin,
            movement::IsFriendlyUnitInFront,
            retreat::{AIRetreatPlugin, KingInSightRange},
        },
        entities::{ProjectileRange, Sight, Unit},
    },
};

use super::{
    entities::{MeleeRange, health::Health},
    physics::PushBack,
};

mod attack;
mod bandit;
mod movement;
mod retreat;

#[derive(Debug, Deref, DerefMut, Component, Default)]
pub struct FollowOffset(pub Vec2);

#[derive(Debug, Component, Default, Clone)]
#[require(FollowOffset)]
pub enum UnitBehaviour {
    #[default]
    FollowFlag,
    Idle,
    Attack(WorldDirection),
}

#[derive(Debug, Component, Default, Clone)]
pub enum BanditBehaviour {
    #[default]
    Aggressive,
}

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BehavePlugin::default(),
            AIAttackPlugin,
            AIMovementPlugin,
            AIRetreatPlugin,
            AIBanditPlugin,
        ))
        .add_observer(on_insert_unit_behaviour)
        .add_observer(push_back_check)
        .add_observer(determine_target)
        .add_observer(check_target_in_melee_range)
        .add_observer(check_target_in_projectile_range)
        .add_systems(FixedPostUpdate, remove_target_if_out_of_sight);

        app.add_systems(Update, debug_display_unit_names);
    }
}

#[derive(Component, Clone)]
enum Attack {
    Melee,
    Projectile,
}

#[derive(Component, Clone)]
struct WalkIntoRange;

#[derive(Component, Clone)]
struct TargetInSightRange;

#[derive(Component, Clone)]
struct RetreatToBase;

#[derive(Component, Clone, Deref)]
struct WalkingInDirection(WorldDirection);

#[derive(Component, Clone, Deref)]
#[relationship(relationship_target = BehaveSources)]
pub struct BehaveTarget(Entity);

#[derive(Component, Clone, Deref)]
#[relationship_target(relationship = BehaveTarget)]
pub struct BehaveSources(Vec<Entity>);

#[derive(Event, Clone)]
struct BeingPushed;

#[derive(Event, Clone)]
struct DetermineTarget;

#[derive(Event, Clone)]
struct TargetInMeleeRange;

#[derive(Event, Clone)]
struct TargetInProjectileRange;

#[derive(Component, Clone, Deref)]
struct WaitToAttack(WorldDirection);

#[derive(Component, Deref)]
#[relationship(relationship_target = TargetedBy)]
pub struct Target(Entity);

#[derive(Component, Deref)]
#[relationship_target(relationship = Target)]
pub struct TargetedBy(Vec<Entity>);

fn on_insert_unit_behaviour(
    trigger: On<Insert, UnitBehaviour>,
    units: Query<&Unit>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.entity;
    let unit = units.get(entity)?;

    let king_within_range = behave!(
        Behave::Sequence =>{
            Behave::trigger(KingInSightRange),
            Behave::spawn_named("Retreat to Base", (RetreatToBase, BehaveTarget(entity)))
        }
    );

    let enemy_within_attack_range = behave!(
        Behave::Sequence => {
            Behave::trigger(TargetInMeleeRange),
            Behave::spawn_named(
                "Attack nearest enemy Melee",
                (
                    Attack::Melee,
                    BehaveInterrupt::by(TargetInProjectileRange).or_not(TargetInMeleeRange),
                    BehaveTarget(entity),
                ),
            )
        }
    );

    let enemy_within_sight_range = if unit.unit_type.ne(&UnitType::Commander) {
        behave!(
            Behave::Sequence => {
                Behave::trigger(TargetInSightRange),
                Behave::Invert => {
                    Behave::trigger(IsFriendlyUnitInFront)
                },
                Behave::spawn_named(
                    "Attack nearest enemy Melee",
                    (
                        WalkIntoRange,
                        BehaveInterrupt::by(TargetInProjectileRange).or(IsFriendlyUnitInFront).or_not(TargetInMeleeRange),
                        BehaveTarget(entity),
                    ),
                ),
            }
        )
    } else {
        behave!(
            Behave::Sequence => {
                Behave::trigger(TargetInSightRange),
                Behave::spawn_named(
                    "Commander",
                    (
                        BehaveTarget(entity),
                    ),
                )
            }
        )
    };

    let tree = behave!(
        Behave::Forever => {
            Behave::Fallback => {
                        @king_within_range,
                        @enemy_within_attack_range,
                        @enemy_within_sight_range,
                        @behave!(
                            Behave::spawn_named(
                                "Following flag",
                                    (FollowFlag, BehaveTarget(entity), BehaveInterrupt::by(TargetInSightRange).or(BeingPushed).or(IsFriendlyUnitInFront))
                            )
                        )
            }
        }
    );

    commands
        .entity(entity)
        .despawn_related::<BehaveSources>()
        .with_child((
            BehaveTree::new(tree).with_logging(false),
            BehaveTarget(entity),
        ));

    Ok(())
}

fn attack_and_walk_in_range(entity: Entity) -> Tree<Behave> {
    behave!(
        Behave::Sequence => {
            Behave::trigger(DetermineTarget),
            Behave::Fallback => {
                Behave::Sequence => {
                    Behave::trigger(TargetInMeleeRange),
                    Behave::spawn_named(
                        "Attack nearest enemy Melee",
                        (
                            Attack::Melee,
                            BehaveInterrupt::by_not(TargetInMeleeRange),
                            BehaveTarget(entity)
                        ),
                    ),
                },
                Behave::spawn_named(
                    "Walking to target",
                    (
                        WalkIntoRange,
                        BehaveInterrupt::by(TargetInMeleeRange),
                        BehaveTimeout::from_secs(2.0, false),
                        BehaveTarget(entity)
                    )
                )
            }
        }
    )
}

fn push_back_check(
    trigger: On<BehaveTrigger<BeingPushed>>,
    query: Query<Option<&PushBack>>,
    mut commands: Commands,
) -> Result {
    let ctx = trigger.event().ctx();
    let maybe_pushback = query.get(ctx.target_entity())?;

    match maybe_pushback {
        Some(push_back) => {
            if push_back.timer.is_finished() {
                commands.trigger(ctx.failure());
            } else {
                commands.trigger(ctx.success());
            }
        }
        None => commands.trigger(ctx.failure()),
    }
    Ok(())
}

fn determine_target(
    trigger: On<BehaveTrigger<TargetInSightRange>>,
    query: Query<(&Transform, &Owner, &Sight, Option<&Target>)>,
    others: Query<(Entity, &Transform, &Owner), With<Health>>,
    mut commands: Commands,
) -> Result {
    let ctx = trigger.event().ctx();
    let unit_entity = ctx.target_entity();
    let (transform, owner, sight, maybe_target) = query.get(unit_entity)?;

    if maybe_target.is_some() {
        commands.trigger(ctx.success());
        return Ok(());
    }

    let nearest = others
        .iter()
        .filter(|(.., other_owner)| other_owner.ne(&owner))
        .map(|(other_entity, other_transform, _)| {
            (
                other_entity,
                transform
                    .translation
                    .truncate()
                    .distance(other_transform.translation.truncate()),
            )
        })
        .filter(|(.., distance)| *distance <= **sight)
        .min_by(|(.., a), (.., b)| a.total_cmp(b));

    match nearest {
        Some((nearest_enemy, ..)) => {
            commands.entity(unit_entity).insert(Target(nearest_enemy));
            commands.trigger(ctx.success());
        }
        None => commands.trigger(ctx.failure()),
    }
    Ok(())
}

fn check_target_in_melee_range(
    trigger: On<BehaveTrigger<TargetInMeleeRange>>,
    query: Query<(&Transform, &MeleeRange, &Target)>,
    transform_query: Query<&Transform>,
    mut commands: Commands,
) -> Result {
    let ctx = trigger.ctx();

    let Ok((transform, range, target)) = query.get(ctx.target_entity()) else {
        commands.trigger(ctx.failure());
        return Ok(());
    };
    let other_transform = transform_query.get(**target)?;
    let distance = transform
        .translation
        .truncate()
        .distance(other_transform.translation.truncate());

    if distance <= **range {
        commands.trigger(ctx.success());
    } else {
        commands.trigger(ctx.failure());
    }
    Ok(())
}

fn check_target_in_projectile_range(
    trigger: On<BehaveTrigger<TargetInProjectileRange>>,
    query: Query<(&Transform, &ProjectileRange, &Target)>,
    transform_query: Query<&Transform>,
    mut commands: Commands,
) -> Result {
    let ev = trigger.event();
    let ctx = ev.ctx();
    let Ok((transform, range, target)) = query.get(ctx.target_entity()) else {
        commands.trigger(ctx.failure());
        return Ok(());
    };

    let other_transform = transform_query.get(**target)?;
    let distance = transform
        .translation
        .truncate()
        .distance(other_transform.translation.truncate());

    if distance <= **range {
        commands.trigger(ctx.success());
    } else {
        commands.trigger(ctx.failure());
    }
    Ok(())
}

fn remove_target_if_out_of_sight(
    query: Query<(Entity, &Target, &Transform, &Sight)>,
    other: Query<&Transform>,
    mut commands: Commands,
) -> Result {
    for (entity, target, transform, sight) in query.iter() {
        let other_transform = other.get(**target)?;
        let distance = transform
            .translation
            .truncate()
            .distance(other_transform.translation.truncate());
        if distance > **sight {
            commands.entity(entity).try_remove::<Target>();
        }
    }
    Ok(())
}

#[derive(Component)]
pub struct DebugNameTag;

#[cfg(debug_assertions)]
fn debug_display_unit_names(
    mut commands: Commands,
    query: Query<(Entity, &Name), (With<Unit>, Without<DebugNameTag>)>,
) {
    let mut y_offset = 1.;
    for (entity, name) in query.iter() {
        y_offset += 4.;
        commands
            .entity(entity)
            .insert(DebugNameTag)
            .with_children(|parent| {
                parent.spawn((
                    Text2d::new(name.as_str()),
                    TextFont {
                        font_size: 4.0,
                        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        ..Default::default()
                    },
                    Transform::from_xyz(0.0, 24.0 + y_offset, 0.0).with_scale(vec3(-1., 1., 1.)),
                ));
            });
    }
}

#[cfg(debug_assertions)]
fn debug_display_targeted_by(
    mut commands: Commands,
    query: Query<(Entity, &TargetedBy), (With<Unit>)>,
    name_query: Query<&Name>,
) -> Result {
    for (entity, targeted_by) in query.iter() {
        let mut targeter_names = Vec::new();
        for targeter in targeted_by.iter() {
            if let Ok(name) = name_query.get(targeter) {
                targeter_names.push(name.as_str().to_string());
            } else {
                targeter_names.push(format!("Entity({:?})", targeter));
            }
        }
        let display_text = if targeter_names.is_empty() {
            "Not targeted".to_string()
        } else {
            format!("Targeted by: {}", targeter_names.join(", "))
        };
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(display_text),
                TextFont {
                    font_size: 4.0,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                    ..Default::default()
                },
                Transform::from_xyz(0.0, 24.0, 0.0).with_scale(vec3(-1., 1., 1.)),
            ));
        });
    }
    Ok(())
}
