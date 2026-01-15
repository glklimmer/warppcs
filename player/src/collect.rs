use bevy::prelude::*;

use interaction::{InteractionTriggeredEvent, InteractionType};
use inventory::Inventory;

pub(crate) struct CollectPlugin;

impl Plugin for CollectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            collect.run_if(on_message::<InteractionTriggeredEvent>),
        );
    }
}

fn collect(
    mut interactions: MessageReader<InteractionTriggeredEvent>,
    mut inventory: Query<&mut Inventory>,
) -> Result {
    for event in interactions.read() {
        let InteractionType::Collect = &event.interaction else {
            continue;
        };

        let mut target_inventory = inventory.get_mut(event.interactable)?;
        let amount = target_inventory.gold;
        target_inventory.gold -= amount;

        let mut player_inventory = inventory.get_mut(event.player)?;

        player_inventory.gold += amount;

        info!("new player inventory: {}", player_inventory.gold);
    }
    Ok(())
}
