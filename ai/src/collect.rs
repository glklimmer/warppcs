use bevy::prelude::*;

use bevy_behave::prelude::BehaveCtx;
use inventory::Inventory;

use crate::{CollectFromEntity, DepositToEntity};

const TRANSFER_AMOUNT: u16 = 50;
const TRANSFER_TICK_SECONDS: f32 = 0.5;

pub(crate) struct CollectPlugin;

impl Plugin for CollectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (collect_gold, deposit_gold));
    }
}

#[derive(Component)]
struct Collecting {
    timer: Timer,
}

#[derive(Component)]
struct Depositing {
    timer: Timer,
}

fn collect_gold(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &CollectFromEntity,
        &BehaveCtx,
        Option<&mut Collecting>,
    )>,
    mut inventories: Query<&mut Inventory>,
) -> Result {
    for (behavior_entity, collect_from, ctx, collecting_opt) in query.iter_mut() {
        let ai_unit_entity = ctx.target_entity();
        let source_entity = **collect_from;

        let Some(mut collecting) = collecting_opt else {
            commands.entity(behavior_entity).insert(Collecting {
                timer: Timer::from_seconds(TRANSFER_TICK_SECONDS, TimerMode::Repeating),
            });
            continue;
        };

        collecting.timer.tick(time.delta());

        if collecting.timer.just_finished() {
            let [mut collector_inv, mut source_inv] =
                inventories.get_many_mut([ai_unit_entity, source_entity])?;

            if source_inv.gold == 0 {
                commands.entity(behavior_entity).remove::<Collecting>();
                commands.trigger(ctx.success());
                continue;
            }

            let amount_to_transfer = TRANSFER_AMOUNT.min(source_inv.gold);
            source_inv.gold -= amount_to_transfer;
            collector_inv.gold += amount_to_transfer;
        }
    }
    Ok(())
}

fn deposit_gold(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &DepositToEntity,
        &BehaveCtx,
        Option<&mut Depositing>,
    )>,
    mut inventories: Query<&mut Inventory>,
) -> Result {
    for (behavior_entity, deposit_to, ctx, depositing_opt) in query.iter_mut() {
        let ai_unit_entity = ctx.target_entity();
        let target_entity = **deposit_to;

        let Some(mut depositing) = depositing_opt else {
            commands.entity(behavior_entity).insert(Depositing {
                timer: Timer::from_seconds(TRANSFER_TICK_SECONDS, TimerMode::Repeating),
            });
            continue;
        };

        depositing.timer.tick(time.delta());

        if depositing.timer.just_finished() {
            let [mut depositor_inv, mut target_inv] =
                inventories.get_many_mut([ai_unit_entity, target_entity])?;

            if depositor_inv.gold == 0 {
                commands.entity(behavior_entity).remove::<Depositing>();
                commands.trigger(ctx.success());
                continue;
            }

            let amount_to_transfer = TRANSFER_AMOUNT.min(depositor_inv.gold);
            depositor_inv.gold -= amount_to_transfer;
            target_inv.gold += amount_to_transfer;
        }
    }
    Ok(())
}
