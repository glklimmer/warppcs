use bevy::prelude::*;

use bevy_behave::{
    behave,
    prelude::{
        Behave, BehaveInterrupt, BehavePlugin, BehaveTimeout, BehaveTree, BehaveTrigger, Tree,
    },
};
use health::Health;
use movement::{
    AIMovementPlugin, FollowFlag, IsFriendlyFormationUnitInFront, IsFriendlyUnitInFront,
};
use physics::WorldDirection;
use shared::Owner;
use units::{MeleeRange, ProjectileRange, Sight, Unit, UnitType, pushback::PushBack};

#[derive(Component, Deref)]
#[relationship(relationship_target = ArmyFormations)]
pub struct ArmyFormationTo(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = ArmyFormationTo, linked_spawn)]
pub struct ArmyFormations(Vec<Entity>);

use crate::{
    attack::AIAttackPlugin,
    bandit::AIBanditPlugin,
    commander::AICommanderPlugin,
    death::DeathPlugin,
    flag::FlagPlugin,
    offset::{FollowOffset, OffsetPlugin},
    retreat::{AIRetreatPlugin, GeneralInSightRange},
    spawn::SpawnPlugin,
};

pub mod offset;
pub mod retreat;

mod attack;
mod bandit;
mod commander;
mod death;
mod flag;
mod movement;
mod spawn;

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
            DeathPlugin,
            SpawnPlugin,
            OffsetPlugin,
            FlagPlugin,
            AIBanditPlugin,
            AICommanderPlugin,
            AIRetreatPlugin,
        ))
        .add_observer(on_insert_unit_behaviour)
        .add_observer(push_back_check)
        .add_observer(determine_target)
        .add_observer(check_target_in_melee_range)
        .add_observer(check_target_in_projectile_range)
        .add_systems(FixedPostUpdate, remove_target_if_out_of_sight);

        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            (debug_display_unit_names, debug_display_targeted_by),
        );
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

#[derive(Component, Clone)]
struct WalkingInDirection;

#[derive(Component, Clone)]
struct Reposition;

#[derive(Component, Clone)]
struct Waiting;

#[derive(Component, Clone)]
pub struct RepositionTo {
    pub x_pos: f32,
}

#[derive(Event, Clone)]
struct FormationHasTarget;

fn on_insert_unit_behaviour(
    trigger: On<Insert, UnitBehaviour>,
    units: Query<&Unit>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.entity;
    let unit = units.get(entity)?;

    if unit.unit_type.eq(&UnitType::Commander) {
        return Ok(());
    }

    let general_within_range = behave!(
        Behave::Sequence =>{
            Behave::trigger(GeneralInSightRange),
            Behave::spawn_named("Retreat to Base", (RetreatToBase, BehaveTarget(entity)))
        }
    );

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
                    BehaveInterrupt::by_not(TargetInProjectileRange)
                        .or(TargetInMeleeRange),
                    BehaveTarget(entity),
                ),
            )
        }
        ));
    }

    let reposition = behave!(
        Behave::Sequence => {
            Behave::trigger(IsFriendlyUnitInFront),
                Behave::spawn_named(
                    "Reposition",
                    (
                        Reposition,
                        BehaveInterrupt::by_not(TargetInProjectileRange)
                            .or(TargetInMeleeRange),
                        BehaveTarget(entity),
                    ),
                ),
        }
    );

    let waiting = behave!(
            Behave::Sequence => {
                Behave::trigger(IsFriendlyFormationUnitInFront),
                Behave::spawn_named(
                    "Waiting",
                    (
                        Waiting,
                        BehaveInterrupt::by_not(IsFriendlyFormationUnitInFront).or_not(TargetInProjectileRange).or_not(TargetInMeleeRange),
                        BehaveTarget(entity),
                    ),
            ),
        }
    );

    let enemy_within_sight_range = behave!(
        Behave::Sequence => {
            Behave::trigger(TargetInSightRange),
            Behave::spawn_named(
                "Walking towards enemy",
                (
                    WalkIntoRange,
                    BehaveInterrupt::by(TargetInProjectileRange).or_not(IsFriendlyFormationUnitInFront).or_not(TargetInMeleeRange),
                    BehaveTarget(entity),
                ),
            ),
        },
    );

    let notify = behave!(
        Behave::Sequence => {
            Behave::trigger(FormationHasTarget),
            Behave::spawn_named(
                "Notify Formation",
                (
                    WalkingInDirection,
                    BehaveInterrupt::by(TargetInProjectileRange).or_not(TargetInMeleeRange),
                    BehaveTarget(entity),
                ),
            ),
        }
    );

    let tree = behave!(
        Behave::Forever => {
            Behave::Fallback => {
                // @general_within_range,
                ...attack_chain,
                @waiting,
                @reposition,
                @enemy_within_sight_range,
                @notify,
                @behave!(
                    Behave::spawn_named(
                        "Following flag",
                            (FollowFlag, BehaveTarget(entity), BehaveInterrupt::by(TargetInSightRange).or(BeingPushed).or(FormationHasTarget))
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

#[derive(Component)]
pub struct DebugTargetedByText;

#[cfg(debug_assertions)]
use bevy::text::{FontSmoothing, LineHeight};
fn debug_display_unit_names(
    mut commands: Commands,
    query: Query<(Entity, &Name), (With<Unit>, Without<DebugNameTag>)>,
    asset_server: Res<AssetServer>,
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
                        font_size: 5.0,
                        line_height: LineHeight::RelativeToFont(1.2),
                        font_smoothing: FontSmoothing::None,
                        font: asset_server.load("fonts/GoogleSansCode-Regular.ttf"),
                    },
                    Transform::from_xyz(0.0, 24.0 + y_offset, 0.0).with_scale(vec3(1., 1., 1.)),
                ));
            });
    }
}

#[cfg(debug_assertions)]
fn debug_display_targeted_by(
    mut commands: Commands,
    unit_query: Query<(Entity, &TargetedBy, Option<&Children>, &Name), With<Unit>>,
    mut text_query: Query<(&mut Text2d, &mut Transform), With<DebugTargetedByText>>,
    name_query: Query<&Name>,
    asset_server: Res<AssetServer>,
) -> Result {
    let mut y_offset = 1.;
    for (unit_entity, targeted_by, children, name) in unit_query.iter() {
        y_offset += 4.;
        let display_text = {
            let mut targeter_names = Vec::new();
            for targeter in targeted_by.iter() {
                if let Ok(name) = name_query.get(targeter) {
                    targeter_names.push(name.as_str().to_string());
                } else {
                    targeter_names.push(format!("Entity({:?})", targeter));
                }
            }
            if targeter_names.is_empty() {
                "Not targeted".to_string()
            } else {
                format!("{} by: {}", name, targeter_names.join(", "))
            }
        };

        let mut text_child_entity = None;
        if let Some(children) = children {
            for child in children.iter() {
                if text_query.get(child).is_ok() {
                    text_child_entity = Some(child);
                    break;
                }
            }
        }

        if let Some(text_child_entity) = text_child_entity {
            if let Ok((mut text, mut transform)) = text_query.get_mut(text_child_entity) {
                text.0 = display_text;
                transform.translation.y = -22.0 - y_offset;
            }
        } else {
            commands.entity(unit_entity).with_children(|parent| {
                parent.spawn((
                    DebugTargetedByText,
                    Text2d::new(display_text),
                    TextColor(Color::linear_rgb(1., 0., 0.)),
                    TextFont {
                        font_size: 5.0,
                        line_height: LineHeight::RelativeToFont(1.2),
                        font_smoothing: FontSmoothing::None,
                        font: asset_server.load("fonts/GoogleSansCode-Regular.ttf"),
                    },
                    Transform::from_xyz(0.0, -22.0 - y_offset, 0.0).with_scale(vec3(1., 1., 1.)),
                ));
            });
        }
    }
    Ok(())
}
