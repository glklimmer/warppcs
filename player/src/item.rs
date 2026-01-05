use bevy::prelude::*;

use interaction::{InteractionTriggeredEvent, InteractionType};
use inventory::Inventory;
use items::Item;

pub(crate) fn pickup_item(
    mut interactions: MessageReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    mut player: Query<&mut Inventory>,
    item: Query<&Item>,
) -> Result {
    for event in interactions.read() {
        let InteractionType::Item = &event.interaction else {
            continue;
        };

        let item = item.get(event.interactable)?;
        let mut inventory = player.get_mut(event.player)?;
        inventory.items.push(item.clone());

        commands.entity(event.interactable).despawn();
    }
    Ok(())
}
