use bevior_tree::prelude::*;
use bevy::prelude::*;

use attack::{unit_range, AttackPlugin};

use crate::{
    map::GameSceneId,
    networking::{MultiplayerRoles, Owner, UnitType},
    GameState,
};

use super::entities::{health::Health, Unit};

pub mod attack;

#[derive(Debug, Component)]
pub enum UnitBehaviour {
    FollowFlag(Entity, Vec2),
    MoveTarget(Vec2),
    AttackTarget(Entity),
    Idle,
}

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BehaviorTreePlugin::default());
        app.add_plugins(AttackPlugin);

        app.add_systems(
            FixedUpdate, // TODO: This should be less then update
            look_for_nearest_target.run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
    }
}

pub const SIGHT_RANGE: f32 = 800.;
pub const MOVE_EPSILON: f32 = 1.;

pub fn unit_tree(unit_type: &UnitType, flag: Entity) -> BehaviorTreeBundle {
    BehaviorTreeBundle::from_root(ConditionalLoop::new(
        Selector::new(vec![
            Box::new(Sequence::new(vec![
                Box::new(EnemyWithinRange::new(unit_range(unit_type))),
                Box::new(Attack::new(unit_range(unit_type))),
            ])),
            Box::new(Sequence::new(vec![
                Box::new(EnemyWithinRange::new(SIGHT_RANGE)),
                Box::new(Approach::new(unit_range(unit_type))),
            ])),
            Box::new(Sequence::new(vec![Box::new(FollowFlag::new(flag))])),
        ]),
        |In(_)| true,
    ))
}

#[derive(Debug, Component, Clone)]
struct TargetInfo {
    entity: Entity,
    distance: f32,
    translation: Vec2,
}

fn look_for_nearest_target(
    mut commands: Commands,
    query: Query<(Entity, &GameSceneId, &Transform, &Owner, &Unit)>,
    others: Query<(Entity, &GameSceneId, &Transform, &Owner), With<Health>>,
) {
    for (entity, scene_id, transform, owner, unit) in query.iter() {
        let maybe_nearest_target = others
            .iter()
            .filter(|other| other.1.eq(scene_id))
            .filter(|other| other.3.ne(owner))
            .map(|other| TargetInfo {
                entity: other.0,
                distance: transform
                    .translation
                    .truncate()
                    .distance(other.2.translation.truncate()),
                translation: other.2.translation.truncate(),
            })
            .filter(|other| other.distance <= unit_range(&unit.unit_type))
            .min_by(|a, b| a.distance.total_cmp(&b.distance));
        match maybe_nearest_target {
            Some(nearast_target) => commands.entity(entity).insert(nearast_target),
            None => commands.entity(entity).remove::<TargetInfo>(),
        };
    }
}

#[delegate_node(delegate)]
struct EnemyWithinRange {
    delegate: TaskBridge,
}
impl EnemyWithinRange {
    pub fn new(range: f32) -> Self {
        let checker = move |In(entity): In<Entity>, query: Query<Option<&TargetInfo>>| {
            let maybe_target = query.get(entity).unwrap();

            match maybe_target {
                Some(target) => match target.distance <= range {
                    true => TaskStatus::Complete(NodeResult::Success),
                    false => TaskStatus::Complete(NodeResult::Failure),
                },
                None => TaskStatus::Complete(NodeResult::Failure),
            }
        };
        Self {
            delegate: TaskBridge::new(checker),
        }
    }
}

#[delegate_node(delegate)]
struct Attack {
    delegate: TaskBridge,
}
impl Attack {
    pub fn new(unit_attack_range: f32) -> Self {
        let checker = move |In(entity): In<Entity>, query: Query<Option<&TargetInfo>>| {
            let maybe_target = query.get(entity).unwrap();

            match maybe_target {
                Some(target) => match target.distance <= unit_attack_range {
                    true => TaskStatus::Running,
                    false => TaskStatus::Complete(NodeResult::Failure),
                },
                None => TaskStatus::Complete(NodeResult::Success),
            }
        };
        let task = TaskBridge::new(checker)
            .on_event(
                TaskEvent::Enter,
                move |In(entity), mut commands: Commands, query: Query<Option<&TargetInfo>>| {
                    let maybe_target = query.get(entity).unwrap();
                    if let Some(target) = maybe_target {
                        commands
                            .entity(entity)
                            .insert(UnitBehaviour::AttackTarget(target.entity));
                    }
                },
            )
            .on_event(TaskEvent::Exit, |In(entity), mut commands: Commands| {
                commands.entity(entity).remove::<UnitBehaviour>();
            });
        Self { delegate: task }
    }
}

#[delegate_node(delegate)]
struct Approach {
    delegate: TaskBridge,
}
impl Approach {
    pub fn new(unit_attack_range: f32) -> Self {
        let checker = move |In(entity): In<Entity>, query: Query<Option<&TargetInfo>>| {
            let maybe_target = query.get(entity).unwrap();

            match maybe_target {
                Some(target) => match target.distance > unit_attack_range {
                    true => TaskStatus::Running,
                    false => TaskStatus::Complete(NodeResult::Success),
                },
                None => TaskStatus::Complete(NodeResult::Failure),
            }
        };
        let task = TaskBridge::new(checker)
            .on_event(
                TaskEvent::Enter,
                move |In(entity), mut commands: Commands, query: Query<Option<&TargetInfo>>| {
                    let maybe_target = query.get(entity).unwrap();
                    if let Some(target) = maybe_target {
                        commands
                            .entity(entity)
                            .insert(UnitBehaviour::MoveTarget(target.translation));
                    }
                },
            )
            .on_event(TaskEvent::Exit, |In(entity), mut commands: Commands| {
                commands.entity(entity).remove::<UnitBehaviour>();
            });
        Self { delegate: task }
    }
}

#[delegate_node(delegate)]
struct FollowFlag {
    delegate: TaskBridge,
}
impl FollowFlag {
    pub fn new(flag: Entity) -> Self {
        let checker = move |In(entity): In<Entity>, query: Query<&Transform>| {
            let unit_transform = query.get(entity).unwrap();
            let flag = query.get(flag).unwrap();

            match flag
                .translation
                .truncate()
                .distance(unit_transform.translation.truncate())
                > MOVE_EPSILON
            {
                true => TaskStatus::Running,
                false => TaskStatus::Complete(NodeResult::Success),
            }
        };
        let task = TaskBridge::new(checker)
            .on_event(
                TaskEvent::Enter,
                move |In(entity), mut commands: Commands, query: Query<Option<&TargetInfo>>| {
                    let maybe_target = query.get(entity).unwrap();
                    if let Some(target) = maybe_target {
                        commands
                            .entity(entity)
                            .insert(UnitBehaviour::FollowFlag(flag, target.translation));
                    }
                },
            )
            .on_event(TaskEvent::Exit, |In(entity), mut commands: Commands| {
                commands.entity(entity).remove::<UnitBehaviour>();
            });
        Self { delegate: task }
    }
}

// fn determine_behaviour(
//     mut query: Query<(
//         Entity,
//         &mut UnitBehaviour,
//         &GameSceneId,
//         &Transform,
//         &Owner,
//         &Unit,
//     )>,
//     others: Query<(Entity, &GameSceneId, &Transform, &Owner), With<Health>>,
//     flag: Query<&FlagAssignment>,
// ) {
//     for (entity, mut behaviour, scene_id, transform, owner, unit) in &mut query {
//         let possible_targets: Vec<TargetInfo> = others
//             .iter()
//             .filter(|other| other.1.eq(scene_id))
//             .filter(|other| other.3.ne(owner))
//             .map(|other| TargetInfo {
//                 entity: other.0,
//                 distance: transform
//                     .translation
//                     .truncate()
//                     .distance(other.2.translation.truncate()),
//                 translation: other.2.translation.truncate(),
//             })
//             .collect();
//
//         let possible_nearest_enemy = possible_targets
//             .iter()
//             .filter(|other| other.distance <= unit_range(&unit.unit_type))
//             .min_by(|a, b| a.distance.total_cmp(&b.distance));
//
//         match possible_nearest_enemy {
//             Some(nearest_enemy) => match *behaviour {
//                 UnitBehaviour::AttackTarget(enemy) => {
//                     if nearest_enemy.entity != enemy {
//                         *behaviour = UnitBehaviour::AttackTarget(nearest_enemy.entity);
//                     }
//                 }
//                 _ => {
//                     *behaviour = UnitBehaviour::AttackTarget(nearest_enemy.entity);
//                 }
//             },
//             None => {
//                 let possible_enemy_in_sight = possible_targets
//                     .iter()
//                     .filter(|other| other.distance <= SIGHT_RANGE)
//                     .min_by(|a, b| a.distance.total_cmp(&b.distance));
//
//                 match possible_enemy_in_sight {
//                     Some(enemy_in_sight) => match *behaviour {
//                         UnitBehaviour::MoveTarget(target) => {
//                             if enemy_in_sight.translation != target {
//                                 *behaviour = UnitBehaviour::MoveTarget(enemy_in_sight.translation);
//                             }
//                         }
//                         _ => {
//                             *behaviour = UnitBehaviour::MoveTarget(enemy_in_sight.translation);
//                         }
//                     },
//                     None => {
//                         let flag = flag.get(entity).unwrap();
//                         match *behaviour {
//                             UnitBehaviour::MoveTarget(target) => {
//                                 if transform.translation.truncate().distance(target) <= MOVE_EPSILON
//                                 {
//                                     *behaviour = UnitBehaviour::FollowFlag(flag.0, flag.1);
//                                 }
//                             }
//                             UnitBehaviour::AttackTarget(_) => {
//                                 *behaviour = UnitBehaviour::FollowFlag(flag.0, flag.1)
//                             }
//                             _ => {}
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
