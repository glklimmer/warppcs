use bevy::prelude::*;

use crate::networking::{Item, UnitType};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_initial_items);
    }
}

#[derive(Resource)]
pub struct ItemPool {
    items: Vec<Item>,
}

impl ItemPool {
    pub fn get_random_item(&self) -> &Item {
        let index = fastrand::usize(..self.items.len());
        &self.items[index]
    }
}

fn add_initial_items(mut commands: Commands) {
    commands.insert_resource(ItemPool {
        items: vec![
            Item {
                name: "Archer",
                tooltip: "Archer",
                effects_unit: UnitType::Archer,
            },
            Item {
                name: "Warrior",
                tooltip: "Warrior",
                effects_unit: UnitType::Shieldwarrior,
            },
            Item {
                name: "Pikeman",
                tooltip: "Pikeman",
                effects_unit: UnitType::Pikeman,
            },
        ],
    });
}
