use bevy::prelude::*;

use bevy_replicon::prelude::{AppRuleExt, Replicated};
use items::Item;
use serde::{Deserialize, Serialize};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<Inventory>();
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone, Default)]
#[require(Replicated)]
pub struct Inventory {
    pub gold: u16,
    pub items: Vec<Item>,
}

pub struct Cost {
    pub gold: u16,
}
