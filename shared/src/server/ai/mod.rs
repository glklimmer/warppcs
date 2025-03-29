use bevy::prelude::*;

use attack::{unit_range, AttackPlugin};

use crate::Owner;

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
            determine_behaviour,
        );
    }
}

pub const SIGHT_RANGE: f32 = 300.;

struct TargetInfo {
    entity: Entity,
    distance: f32,
}

fn determine_behaviour(
    mut query: Query<(Entity, &mut UnitBehaviour, &Transform, &Owner, &Unit)>,
    others: Query<(Entity, &Transform, &Owner), With<Health>>,
    flag: Query<&FlagAssignment>,
) {
    for (entity, mut behaviour, transform, owner, unit) in &mut query {
        let nearest = others
            .iter()
            .filter(|(.., other_owner)| other_owner.ne(&owner))
            .map(|(other_entity, other_transform, _)| TargetInfo {
                entity: other_entity,
                distance: transform
                    .translation
                    .truncate()
                    .distance(other_transform.translation.truncate()),
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
