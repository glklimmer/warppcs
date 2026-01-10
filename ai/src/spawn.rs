use bevy::prelude::*;

use units::{Unit, UnitType};

use crate::{UnitBehaviour, bandit::BanditBehaviour};

pub(crate) struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_spawn_unit);
    }
}

fn on_spawn_unit(spawn: On<Add, Unit>, query: Query<&Unit>, mut commands: Commands) -> Result {
    let entity = spawn.entity;
    let unit = query.get(entity)?;

    match unit.unit_type {
        UnitType::Shieldwarrior | UnitType::Pikeman | UnitType::Archer | UnitType::Commander => {
            commands.entity(entity).insert(UnitBehaviour::default())
        }
        UnitType::Bandit => commands.entity(entity).insert(BanditBehaviour::default()),
    };

    Ok(())
}
