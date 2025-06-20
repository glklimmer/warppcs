use bevy::prelude::*;

use attack::AIAttackPlugin;
use bevy_behave::{
    Behave, behave,
    prelude::{BehavePlugin, BehaveTimeout, BehaveTree, BehaveTrigger},
};
use movement::AIMovementPlugin;

use crate::{Owner, networking::WorldDirection};

use super::{
    entities::{Range, health::Health},
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

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BehavePlugin::default(), AIAttackPlugin, AIMovementPlugin))
            .add_observer(on_insert_unit_behaviour)
            .add_observer(push_back_check)
            .add_observer(determine_target)
            .add_observer(has_enemy_in_range)
            .add_systems(
                FixedUpdate,
                (remove_target_if_out_of_sight, remove_target_if_out_of_sight),
            );
    }
}

#[derive(Component, Clone)]
struct AttackingInRange;

#[derive(Component, Clone)]
struct WalkIntoRange;

#[derive(Component, Clone)]
struct FollowFlag;

#[derive(Component, Clone, Deref)]
struct WalkingInDirection(WorldDirection);

pub const SIGHT_RANGE: f32 = 300.;

fn on_insert_unit_behaviour(
    trigger: Trigger<OnInsert, UnitBehaviour>,
    mut commands: Commands,
    query: Query<&UnitBehaviour>,
) {
    let entity = trigger.target();
    let behaviour = query.get(entity).unwrap();

    let attack_nearby = match behaviour {
        UnitBehaviour::FollowFlag => {
            behave!(
                Behave::IfThen => {
                    Behave::trigger(DetermineTarget),
                    Behave::IfThen => {
                        Behave::trigger(CheckHasEnemyInRange),
                        Behave::spawn_named(
                            "Attack nearest enemy",
                            (AttackingInRange, BehaveTarget(entity))
                        )
                    }
                }
            )
        }
        _ => {
            behave!(
                Behave::IfThen => {
                    Behave::trigger(DetermineTarget),
                    Behave::IfThen => {
                        Behave::trigger(CheckHasEnemyInRange),
                        Behave::spawn_named(
                            "Attack nearest enemy",
                            (AttackingInRange, BehaveTarget(entity))
                        ),
                        Behave::spawn_named(
                            "Walk to nearest enemy in sight",
                            (WalkIntoRange, BehaveTarget(entity))
                        )
                    }
                }
            )
        }
    };

    let stance = match behaviour {
        UnitBehaviour::Idle | UnitBehaviour::FollowFlag => behave!(Behave::spawn_named(
            "Following flag",
            (
                FollowFlag,
                BehaveTimeout::from_secs(2.0, true),
                BehaveTarget(entity)
            )
        )),
        UnitBehaviour::Attack(direction) => behave!(Behave::spawn_named(
            "Walking in attack direction",
            (
                WalkingInDirection(*direction),
                BehaveTimeout::from_secs(2.0, true),
                BehaveTarget(entity)
            )
        )),
    };

    let tree = behave!(
        Behave::Forever => {
            Behave::Sequence => {
                Behave::trigger(PushBackCheck),
                Behave::Fallback => {
                    @ attack_nearby,
                    @ stance
                }
            }
        }
    );

    info!("new behave: {:?}", behaviour);
    commands
        .entity(entity)
        .despawn_related::<BehaveSources>()
        .with_child((
            BehaveTree::new(tree).with_logging(false),
            BehaveTarget(entity),
        ));
}

#[derive(Component, Clone, Deref)]
#[relationship(relationship_target = BehaveSources)]
pub struct BehaveTarget(Entity);

#[derive(Component, Clone, Deref)]
#[relationship_target(relationship = BehaveTarget)]
pub struct BehaveSources(Vec<Entity>);

#[derive(Clone)]
struct PushBackCheck;

#[derive(Clone)]
struct DetermineTarget;

#[derive(Clone)]
struct CheckHasEnemyInRange;

#[derive(Component, Deref)]
#[relationship(relationship_target = TargetedBy)]
pub struct Target(Entity);

#[derive(Component, Deref)]
#[relationship_target(relationship = Target)]
pub struct TargetedBy(Vec<Entity>);

fn push_back_check(
    trigger: Trigger<BehaveTrigger<PushBackCheck>>,
    mut commands: Commands,
    query: Query<Option<&PushBack>>,
) {
    let ctx = trigger.event().ctx();
    let maybe_pushback = query.get(ctx.target_entity()).unwrap();

    match maybe_pushback {
        Some(push_back) => {
            if push_back.timer.finished() {
                commands.trigger(ctx.success());
            } else {
                commands.trigger(ctx.failure());
            }
        }
        None => commands.trigger(ctx.success()),
    }
}

fn determine_target(
    trigger: Trigger<BehaveTrigger<DetermineTarget>>,
    mut commands: Commands,
    query: Query<(&Transform, &Owner, Option<&Target>)>,
    others: Query<(Entity, &Transform, &Owner), With<Health>>,
) {
    let ctx = trigger.event().ctx();
    let unit_entity = ctx.target_entity();
    let (transform, owner, maybe_target) = query.get(unit_entity).unwrap();

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
        .filter(|(.., distance)| *distance <= SIGHT_RANGE)
        .min_by(|(.., a), (.., b)| a.total_cmp(b));

    match nearest {
        Some((nearest_enemy, ..)) => {
            commands.entity(unit_entity).insert(Target(nearest_enemy));
            commands.trigger(ctx.success());
        }
        None => commands.trigger(ctx.failure()),
    }
}

fn has_enemy_in_range(
    trigger: Trigger<BehaveTrigger<CheckHasEnemyInRange>>,
    mut commands: Commands,
    query: Query<(&Transform, &Range, Option<&Target>)>,
    transform_query: Query<&Transform>,
) {
    let ctx = trigger.ctx();
    let (transform, range, maybe_target) = query.get(ctx.target_entity()).unwrap();
    let Some(target) = maybe_target else {
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
    query: Query<(Entity, &Target, &Transform)>,
    other: Query<&Transform>,
) {
    for (entity, target, transform) in query.iter() {
        let other_transform = other.get(**target).unwrap();
        let distance = transform
            .translation
            .truncate()
            .distance(other_transform.translation.truncate());
        if distance > SIGHT_RANGE {
            commands.entity(entity).try_remove::<Target>();
        }
    }
}
