use bevy::prelude::*;

use units::Unit;

use crate::UnitBehaviour;

pub(crate) struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_spawn_unit);
    }
}

fn on_spawn_unit(spawn: On<Add, Unit>, mut commands: Commands) -> Result {
    let entity = spawn.entity;

    commands.entity(entity).insert(UnitBehaviour::default());

    Ok(())
}
