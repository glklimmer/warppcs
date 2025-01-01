use bevy::prelude::*;

use attack::{unit_range, AttackPlugin};

use crate::{
    map::GameSceneId,
    networking::{MultiplayerRoles, Owner},
    GameState,
};

use super::{
    buildings::recruiting::FlagAssignment,
    entities::{health::Health, Unit},
};

pub mod attack;

#[derive(Debug, Component)]
pub enum UnitBehaviour {
    FollowFlag(Entity, Vec2),
    AttackTarget(Entity),
    Idle,
}

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AttackPlugin);

        app.add_systems(
            FixedUpdate, // TODO: This should be less then update
            determine_behaviour
                .run_if(in_state(GameState::GameSession).and(in_state(MultiplayerRoles::Host))),
        );
    }
}

pub const SIGHT_RANGE: f32 = 800.;

struct TargetInfo {
    entity: Entity,
    distance: f32,
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
    others: Query<(Entity, &GameSceneId, &Transform, &Owner), With<Health>>,
    flag: Query<&FlagAssignment>,
) {
    for (entity, mut behaviour, scene_id, transform, owner, unit) in &mut query {
        let nearest = others
            .iter()
            .filter(|other| other.1.eq(scene_id))
            .filter(|other| other.3.ne(owner))
            .map(|other| TargetInfo {
                entity: other.0,
                distance: transform
                    .translation
                    .truncate()
                    .distance(other.2.translation.truncate()),
            })
            .filter(|other| other.distance <= unit_range(&unit.unit_type))
            .min_by(|a, b| a.distance.total_cmp(&b.distance));

        match nearest {
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
                let flag = flag.get(entity).unwrap();
                *behaviour = UnitBehaviour::FollowFlag(flag.0, flag.1);
            }
        }
    }
}
