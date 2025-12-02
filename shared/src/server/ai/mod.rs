use bevy::prelude::*;
use bevy_behave::prelude::*;

use attack::AIAttackPlugin;
use bevy_behave::{Behave, behave};
use movement::{AIMovementPlugin, FollowFlag, Roam};

use crate::{
    Owner,
    networking::{UnitType, WorldDirection},
    server::entities::{ProjectileRange, Sight, Unit},
};

use super::{
    entities::{MeleeRange, health::Health},
    physics::PushBack,
};

mod attack;
mod movement;

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
        app.add_plugins((BehavePlugin::default(), AIAttackPlugin, AIMovementPlugin))
            .add_observer(on_insert_unit_behaviour)
            .add_observer(on_insert_bandit_behaviour)
            .add_observer(push_back_check)
            .add_observer(determine_target)
            .add_observer(check_target_in_melee_range)
            .add_observer(check_target_in_projectile_range)
            .add_systems(FixedPostUpdate, remove_target_if_out_of_sight);
    }
}

#[derive(Component, Clone)]
enum Attack {
    Melee,
    Projectile,
}

#[derive(Component, Clone)]
struct WalkIntoRange;

#[derive(Component, Clone, Deref)]
struct WalkingInDirection(WorldDirection);

fn on_insert_unit_behaviour(
    trigger: On<Insert, UnitBehaviour>,
    query: Query<(&UnitBehaviour, &Unit)>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.target();
    let (behaviour, unit) = query.get(entity)?;

    let mut attack_chain: Vec<Tree<Behave>> = Vec::new();

    attack_chain.push(behave!(
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
    ));

    if let UnitType::Archer = unit.unit_type {
        attack_chain.push(behave!(
        Behave::Sequence => {
            Behave::trigger(TargetInProjectileRange),
            Behave::spawn_named(
                "Attack nearest enemy Range",
                (
                    Attack::Projectile,
                    BehaveInterrupt::by_not(TargetInProjectileRange).or(TargetInMeleeRange),
                    BehaveTarget(entity),
                ),
            )
        }
        ));
    }

    if let UnitBehaviour::Idle | UnitBehaviour::Attack(_) = behaviour {
        attack_chain.push(behave!(Behave::spawn_named(
            "Walking to target",
            (
                WalkIntoRange,
                BehaveInterrupt::by(TargetInMeleeRange).or(TargetInProjectileRange),
                BehaveTimeout::from_secs(3.0, false),
                BehaveTarget(entity),
            ),
        )));
    }

    let stance = match behaviour {
        UnitBehaviour::Idle | UnitBehaviour::FollowFlag => behave!(Behave::spawn_named(
            "Following flag",
            (
                FollowFlag,
                BehaveInterrupt::by(DetermineTarget).or(BeingPushed),
                BehaveTarget(entity)
            )
        )),
        UnitBehaviour::Attack(direction) => behave!(
            Behave::Sequence => {
                Behave::spawn((
                    Name::new("Wait until unit can attack in direction"),
                    WaitToAttack(*direction)
                )),
                Behave::spawn_named(
                "Attacking direction",
                (
                    WalkingInDirection(*direction),
                    BehaveInterrupt::by(DetermineTarget).or(BeingPushed),
                    BehaveTarget(entity)
                ))
            }
        ),
    };

    let tree = behave!(
        Behave::Forever => {
            Behave::Fallback => {
                Behave::Sequence => {
                    Behave::trigger(DetermineTarget),
                    Behave::Fallback => {
                        ... attack_chain
                    }
                },
                @ stance
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

fn on_insert_bandit_behaviour(
    trigger: On<Insert, BanditBehaviour>,
    query: Query<&BanditBehaviour>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.target();
    let behaviour = query.get(entity)?;

    let stance = match behaviour {
        BanditBehaviour::Aggressive => behave!(Behave::spawn_named(
            "Roaming",
            (
                Roam::default(),
                BehaveInterrupt::by(DetermineTarget).or(BeingPushed),
                BehaveTarget(entity)
            )
        )),
    };

    let attack_chain = attack_and_walk_in_range(entity);

    let tree = behave!(
        Behave::Forever => {
            Behave::Fallback => {
                @ attack_chain,
                @ stance
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

fn push_back_check(
    trigger: On<BehaveTrigger<BeingPushed>>,
    query: Query<Option<&PushBack>>,
    mut commands: Commands,
) -> Result {
    let ctx = trigger.event().ctx();
    let maybe_pushback = query.get(ctx.target_entity())?;

    match maybe_pushback {
        Some(push_back) => {
            if push_back.timer.finished() {
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
    trigger: On<BehaveTrigger<DetermineTarget>>,
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
