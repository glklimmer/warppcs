use bevy::prelude::*;

use bevy_behave::prelude::BehaveTrigger;
use bevy_replicon::prelude::ServerState;
use health::Health;
use units::Unit;

use crate::{
    BehaveSources, EntityDespawn, Target, TargetedBy, UnitBehaviour, bandit::BanditBehaviour,
};

pub(crate) struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_death).add_observer(on_entity_despawn);
    }
}

fn on_death(
    death: On<Remove, Health>,
    units: Query<Option<&TargetedBy>, With<Unit>>,
    server_state: Res<State<ServerState>>,
    mut commands: Commands,
) -> Result {
    let ServerState::Running = server_state.get() else {
        return Ok(());
    };
    let entity = death.entity;
    let Ok(maybe_targeted_by) = units.get(entity) else {
        return Ok(());
    };

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

fn on_entity_despawn(trigger: On<BehaveTrigger<EntityDespawn>>, mut commands: Commands) -> Result {
    let ctx = trigger.event().ctx();
    let target_entity = ctx.target_entity();

    commands.entity(target_entity).despawn();
    commands.trigger(ctx.success());

    Ok(())
}
