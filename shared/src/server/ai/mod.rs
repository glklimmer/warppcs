use bevy::prelude::*;

use attack::{unit_range, AttackPlugin};

use crate::{
    map::GameSceneId,
    networking::{MultiplayerRoles, Owner},
    GameState,
};

use super::{buildings::recruiting::FlagAssignment, entities::Unit};

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
        app.add_plugins(AttackPlugin);

        app.add_systems(
            FixedUpdate, // TODO: This should be less then update
            determine_behaviour.run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
    }
}

pub const SIGHT_RANGE: f32 = 800.;
pub const MOVE_EPSILON: f32 = 1.;

struct TargetInfo {
    entity: Entity,
    distance: f32,
    translation: Vec2,
}

fn determine_behaviour(
    mut query: Query<(
        Entity,
        &mut UnitBehaviour,
        &GameSceneId,
        &Transform,
        &Owner,
        &Unit,
    )>,
    others: Query<(Entity, &GameSceneId, &Transform, &Owner), With<Unit>>,
    flag: Query<&FlagAssignment>,
) {
    for (entity, mut behaviour, scene_id, transform, owner, unit) in &mut query {
        let possible_targets: Vec<TargetInfo> = others
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
            .collect();

        let possible_nearest_enemy = possible_targets
            .iter()
            .filter(|other| other.distance <= unit_range(&unit.unit_type))
            .min_by(|a, b| a.distance.total_cmp(&b.distance));

        match possible_nearest_enemy {
            Some(nearest_enemy) => match *behaviour {
                UnitBehaviour::AttackTarget(enemy) => {
                    if nearest_enemy.entity != enemy {
                        *behaviour = UnitBehaviour::AttackTarget(nearest_enemy.entity);
                    }
                }
                _ => {
                    *behaviour = UnitBehaviour::AttackTarget(nearest_enemy.entity);
                }
            },
            None => {
                let possible_enemy_in_sight = possible_targets
                    .iter()
                    .filter(|other| other.distance <= SIGHT_RANGE)
                    .min_by(|a, b| a.distance.total_cmp(&b.distance));

                match possible_enemy_in_sight {
                    Some(enemy_in_sight) => match *behaviour {
                        UnitBehaviour::MoveTarget(target) => {
                            if enemy_in_sight.translation != target {
                                *behaviour = UnitBehaviour::MoveTarget(enemy_in_sight.translation);
                            }
                        }
                        _ => {
                            *behaviour = UnitBehaviour::MoveTarget(enemy_in_sight.translation);
                        }
                    },
                    None => {
                        let flag = flag.get(entity).unwrap();
                        match *behaviour {
                            UnitBehaviour::MoveTarget(target) => {
                                if transform.translation.truncate().distance(target) <= MOVE_EPSILON
                                {
                                    *behaviour = UnitBehaviour::FollowFlag(flag.0, flag.1);
                                }
                            }
                            UnitBehaviour::AttackTarget(_) => {
                                *behaviour = UnitBehaviour::FollowFlag(flag.0, flag.1)
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
