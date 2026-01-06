use bevy::prelude::*;

use health::Health;

use crate::{BanditBehaviour, BehaveSources, Target, TargetedBy, UnitBehaviour};

pub(crate) struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_death);
    }
}

fn on_death(
    death: On<Remove, Health>,
    units: Query<Option<&TargetedBy>>,
    mut commands: Commands,
) -> Result {
    let entity = death.entity;
    let maybe_targeted_by = units.get(entity)?;

    commands
        .entity(entity)
        .despawn_related::<BehaveSources>()
        .try_remove::<BanditBehaviour>()
        .try_remove::<UnitBehaviour>();

    if let Some(targeted_by) = maybe_targeted_by {
        commands
            .entity(entity)
            .remove_related::<Target>(targeted_by);
    };

    Ok(())
}
