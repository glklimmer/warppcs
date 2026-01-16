use bevy::prelude::*;

use bevy_behave::{
    Behave, behave,
    prelude::{BehaveCtx, BehaveTree},
};
use transport::{HomeBuilding, Transport};

use crate::{CollectFromEntity, DepositToEntity, TravelToEntity};

pub(crate) struct CollectPlugin;

impl Plugin for CollectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (collect_from_entity, deposit_to_entity));
    }
}

fn collect_from_entity(
    query: Query<(&BehaveCtx, &CollectFromEntity)>,
    mut commands: Commands,
) -> Result {
    for (ctx, collect_from_entity) in query.iter() {}
    Ok(())
}

fn deposit_to_entity(
    query: Query<(&BehaveCtx, &DepositToEntity)>,
    mut commands: Commands,
) -> Result {
    for (ctx, deposit_to_entity) in query.iter() {}
    Ok(())
}
