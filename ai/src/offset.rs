use army::flag::{FlagAssignment, FlagUnits};
use bevy::prelude::*;

pub(crate) struct OffsetPlugin;

impl Plugin for OffsetPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(assign_offset);
    }
}

#[derive(Debug, Deref, DerefMut, Component, Default)]
pub struct FollowOffset(pub Vec2);

fn assign_offset(
    trigger: On<Add, FlagAssignment>,
    mut units: Query<&mut FollowOffset>,
    flag_units_query: Query<&FlagUnits>,
    flag_assignment_query: Query<&FlagAssignment>,
) -> Result {
    let flag_assignment = flag_assignment_query.get(trigger.entity)?;
    let flag_entity = **flag_assignment;

    let Ok(flag_units) = flag_units_query.get(flag_entity) else {
        return Ok(());
    };

    let mut unit_entities = (**flag_units).to_vec();
    unit_entities.push(trigger.entity);

    fastrand::shuffle(&mut unit_entities);

    let count = unit_entities.len() as f32;
    let half = (count - 1.0) / 2.0;
    let spacing = 15.0;
    let shift = if unit_entities.len() % 2 == 1 {
        spacing / 2.0
    } else {
        0.0
    };

    for (i, unit_entity) in unit_entities.into_iter().enumerate() {
        if let Ok(mut offset) = units.get_mut(unit_entity) {
            let index = i as f32;
            let offset_x = spacing * (index - half) - shift;
            offset.0 = Vec2::new(offset_x, 0.0);
        }
    }
    Ok(())
}
