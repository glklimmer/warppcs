use bevy::prelude::*;

use bevy_replicon::prelude::ToClients;
use health::{Health, TakeDamage};
use shared::{AnimationChangeEvent, Owner};
use units::Unit;

use crate::{BanditBehaviour, BehaveSources, Target, TargetedBy, UnitBehaviour};

pub(crate) struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_unit_death);
    }
}

fn on_unit_death(
    death: On<Remove, Health>,
    units: Query<(&Health, &Owner, Option<&TargetedBy>), With<Unit>>,
    transform: Query<&Transform>,
    mut commands: Commands,
) -> Result {
    let entity = death.entity();
    let (health, owner, maybe_targeted_by, maybe_flag_assignment, maybe_army) =
        units.get(entity)?;

    commands
        .entity(entity)
        .despawn_related::<BehaveSources>()
        .remove::<BanditBehaviour>()
        .remove::<UnitBehaviour>();

    if let Some(targeted_by) = maybe_targeted_by {
        commands
            .entity(entity)
            .remove_related::<Target>(targeted_by);
    };

    Ok(())
}
