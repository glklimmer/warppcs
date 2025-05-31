use bevy::prelude::*;

use attack::AttackPlugin;

use crate::Owner;

use super::{
    buildings::recruiting::FlagAssignment,
    entities::{Range, health::Health},
};

pub mod attack;

#[derive(Debug, Deref, DerefMut, Component)]
pub struct FollowOffset(pub Vec2);

#[derive(Debug, Component)]
pub enum UnitBehaviour {
    FollowFlag(Entity),
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
    mut query: Query<(Entity, &mut UnitBehaviour, &Transform, &Owner, &Range)>,
    others: Query<(Entity, &Transform, &Owner), With<Health>>,
    flag: Query<&FlagAssignment>,
) {
    for (entity, mut behaviour, transform, owner, range) in &mut query {
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
            .filter(|other| other.distance <= **range)
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
                *behaviour = UnitBehaviour::FollowFlag(flag.0);
            }
        }
    }
}
