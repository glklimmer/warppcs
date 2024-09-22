use attack::{unit_range, AttackPlugin};
use bevy::prelude::*;

use shared::networking::{Owner, Unit};

pub mod attack;

#[derive(Debug, Component)]
pub enum UnitBehaviour {
    MoveTarget(Vec2),
    AttackTarget(Entity),
    Idle,
}

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AttackPlugin);

        app.add_systems(Update, (determine_behaviour,)); // TODO: This should be less then update
    }
}

pub const SIGHT_RANGE: f32 = 800.;

struct TargetInfo {
    entity: Entity,
    distance: f32,
    translation: Vec2,
}

fn determine_behaviour(
    mut query: Query<(&mut UnitBehaviour, &Transform, &Owner, &Unit)>,
    others: Query<(Entity, &Transform, &Owner), With<Unit>>,
) {
    for (mut behaviour, transform, owner, unit) in &mut query {
        let possible_targets: Vec<TargetInfo> = others
            .iter()
            .filter(|other| other.2.ne(owner))
            .map(|other| TargetInfo {
                entity: other.0,
                distance: transform
                    .translation
                    .truncate()
                    .distance(other.1.translation.truncate()),
                translation: other.1.translation.truncate(),
            })
            .collect();

        let possible_nearest_enemy = possible_targets
            .iter()
            .filter(|other| other.distance <= unit_range(&unit.unit_type))
            .min_by(|a, b| a.distance.total_cmp(&b.distance));

        if let Some(nearest_enemy) = possible_nearest_enemy {
            match *behaviour {
                UnitBehaviour::AttackTarget(enemy) => {
                    if nearest_enemy.entity != enemy {
                        *behaviour = UnitBehaviour::AttackTarget(nearest_enemy.entity);
                    }
                }
                _ => {
                    *behaviour = UnitBehaviour::AttackTarget(nearest_enemy.entity);
                }
            }
        } else {
            let possible_enemy_in_sight = possible_targets
                .iter()
                .filter(|other| other.distance <= SIGHT_RANGE)
                .min_by(|a, b| a.distance.total_cmp(&b.distance));

            if let Some(enemy_in_sight) = possible_enemy_in_sight {
                match *behaviour {
                    UnitBehaviour::MoveTarget(target) => {
                        if enemy_in_sight.translation != target {
                            *behaviour = UnitBehaviour::MoveTarget(enemy_in_sight.translation);
                        }
                    }
                    _ => {
                        *behaviour = UnitBehaviour::MoveTarget(enemy_in_sight.translation);
                    }
                }
            }
        }
    }
}
