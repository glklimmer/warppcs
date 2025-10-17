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
            .add_observer(check_target_in_melee_proximity)
            .add_observer(check_target_in_range_proximity)
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
    trigger: Trigger<OnInsert, UnitBehaviour>,
    mut commands: Commands,
    query: Query<(&UnitBehaviour, &Unit)>,
) {
    let entity = trigger.target();
    let (behaviour, unit) = query.get(entity).unwrap();

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
            BehaveTree::new(tree).with_logging(true),
            BehaveTarget(entity),
        ));
}

fn on_insert_bandit_behaviour(
    trigger: Trigger<OnInsert, BanditBehaviour>,
    mut commands: Commands,
    query: Query<&BanditBehaviour>,
) {
    let entity = trigger.target();
    let behaviour = query.get(entity).unwrap();

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

#[derive(Clone)]
struct BeingPushed;

#[derive(Clone)]
struct DetermineTarget;

#[derive(Clone)]
struct TargetInMeleeRange;

#[derive(Clone)]
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
    trigger: Trigger<BehaveTrigger<BeingPushed>>,
    mut commands: Commands,
    query: Query<Option<&PushBack>>,
) {
    let ctx = trigger.event().ctx();
    let maybe_pushback = query.get(ctx.target_entity()).unwrap();

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
}

fn determine_target(
    trigger: Trigger<BehaveTrigger<DetermineTarget>>,
    mut commands: Commands,
    query: Query<(&Transform, &Owner, &Sight, Option<&Target>)>,
    others: Query<(Entity, &Transform, &Owner), With<Health>>,
) {
    let ctx = trigger.event().ctx();
    let unit_entity = ctx.target_entity();
    let (transform, owner, sight, maybe_target) = query.get(unit_entity).unwrap();

    if maybe_target.is_some() {
        commands.trigger(ctx.success());
        return;
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
}

fn check_target_in_melee_proximity(
    trigger: Trigger<BehaveTrigger<TargetInMeleeRange>>,
    mut commands: Commands,
    query: Query<(&Transform, &MeleeRange, &Target)>,
    transform_query: Query<&Transform>,
) {
    let ctx = trigger.ctx();

    let Ok((transform, range, target)) = query.get(ctx.target_entity()) else {
        commands.trigger(ctx.failure());
        return;
    };
    let other_transform = transform_query.get(**target).unwrap();
    let distance = transform
        .translation
        .truncate()
        .distance(other_transform.translation.truncate());

    if distance <= **range {
        commands.trigger(ctx.success());
    } else {
        commands.trigger(ctx.failure());
    }
}

fn check_target_in_range_proximity(
    trigger: Trigger<BehaveTrigger<TargetInProjectileRange>>,
    mut commands: Commands,
    query: Query<(&Transform, &ProjectileRange, &Target)>,
    transform_query: Query<&Transform>,
) {
    let ctx = trigger.ctx();
    let Ok((transform, range, target)) = query.get(ctx.target_entity()) else {
        commands.trigger(ctx.failure());
        return;
    };

    let other_transform = transform_query.get(**target).unwrap();
    let distance = transform
        .translation
        .truncate()
        .distance(other_transform.translation.truncate());

    if distance <= **range {
        commands.trigger(ctx.success());
    } else {
        commands.trigger(ctx.failure());
    }
}

fn remove_target_if_out_of_sight(
    mut commands: Commands,
    query: Query<(Entity, &Target, &Transform, &Sight)>,
    other: Query<&Transform>,
) {
    for (entity, target, transform, sight) in query.iter() {
        let other_transform = other.get(**target).unwrap();
        let distance = transform
            .translation
            .truncate()
            .distance(other_transform.translation.truncate());
        if distance > **sight {
            commands.entity(entity).try_remove::<Target>();
        }
    }
}
