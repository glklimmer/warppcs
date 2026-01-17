use bevy::prelude::*;

use bevy_behave::{Behave, behave, prelude::BehaveTree};
use transport::{HomeBuilding, Transport};

use crate::{CollectFromEntity, DepositToEntity, EntityDespawn, TravelToEntity};

pub(crate) struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_spawn_transport);
    }
}

fn on_spawn_transport(
    spawn: On<Add, Transport>,
    query: Query<(&Transport, &HomeBuilding)>,
    mut commands: Commands,
) -> Result {
    let entity = spawn.entity;
    let (transport, home_building) = query.get(entity)?;

    let tree = behave!(Behave::Sequence => {
        Behave::spawn_named(
            "Traveling to collectable",
            TravelToEntity(transport.target)
        ),
        Behave::spawn_named(
            "Collect",
            CollectFromEntity(transport.target)
        ),
        Behave::spawn_named(
            "Travel back home",
            TravelToEntity(**home_building)
        ),
        Behave::spawn_named(
            "Deposit",
            DepositToEntity(**home_building)
        ),
        Behave::trigger(EntityDespawn)
    });

    commands
        .entity(entity)
        .with_child((BehaveTree::new(tree).with_logging(true),));
    Ok(())
}
