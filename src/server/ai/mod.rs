use attack::{unit_range, AttackPlugin};
use bevy::prelude::*;

use crate::shared::networking::{Owner, Unit};

pub mod attack;

#[derive(Component)]
pub enum UnitBehaviour {
    MoveTarget(Vec3),
    AttackTarget(Entity),
    Idle,
}

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AttackPlugin);

        app.add_systems(Update, determine_behaviour);
    }
}

const SIGHT_RANGE: f32 = 60.;

fn determine_behaviour(
    mut query: Query<(&mut UnitBehaviour, &Transform, &Owner, &Unit)>,
    others: Query<(Entity, &Transform, &Owner), With<Unit>>,
) {
    for (mut behaviour, transform, owner, unit) in &mut query {
        let maybe_nearast_enemy = others
            .iter()
            .filter(|other| other.2.ne(owner))
            .map(|other| {
                (
                    other.0,
                    transform
                        .translation
                        .truncate()
                        .distance(other.1.translation.truncate()),
                )
            })
            .filter(|other| other.1 <= unit_range(&unit.unit_type))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some(nearest_enemy) = maybe_nearast_enemy {
            *behaviour = UnitBehaviour::AttackTarget(nearest_enemy.0);
        }
    }
}
