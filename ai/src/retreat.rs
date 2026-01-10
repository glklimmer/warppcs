use bevy::prelude::*;

use bevy_behave::prelude::{BehaveCtx, BehaveTrigger};
use units::{Sight, Unit};

use crate::{Owner, RetreatToBase};

pub(crate) struct AIRetreatPlugin;

impl Plugin for AIRetreatPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(king_in_sight_range)
            .add_systems(FixedUpdate, retreat_to_base);
    }
}

#[derive(Component, Default)]
pub struct General;

#[derive(Component, Clone)]
pub(crate) struct GeneralInSightRange;

fn king_in_sight_range(
    trigger: On<BehaveTrigger<GeneralInSightRange>>,
    query: Query<(&Transform, &Owner, &Sight)>,
    kings: Query<(&Transform, &Owner), With<General>>,
    mut commands: Commands,
) -> Result {
    let ctx = trigger.event().ctx();
    let unit_entity = ctx.target_entity();
    let (transform, owner, sight) = query.get(unit_entity)?;

    let is_out_of_sight = kings
        .iter()
        .filter(|(_, other_owner)| owner.is_different_faction(other_owner))
        .filter_map(|(king_transform, _)| {
            let distance = transform
                .translation
                .truncate()
                .distance(king_transform.translation.truncate());
            if distance <= **sight {
                Some((king_transform, distance))
            } else {
                None
            }
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    match is_out_of_sight {
        Some((_nearest_transform, _)) => {
            commands.trigger(ctx.success());
        }
        None => commands.trigger(ctx.failure()),
    }
    Ok(())
}

fn retreat_to_base(
    query: Query<&BehaveCtx, With<RetreatToBase>>,
    mut unit: Query<&mut Transform, With<Unit>>,
) {
    for ctx in query.iter() {
        let _unit_post = unit.get_mut(ctx.target_entity());
    }
}
