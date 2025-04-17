use bevy::{prelude::*, utils::HashMap};
use bevy_replicon::prelude::{SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    ClientPlayerMap, Vec3LayerExt,
    enum_map::*,
    map::Layers,
    server::players::{
        interaction::{InteractionTriggeredEvent, InteractionType},
        items::{Item, ItemType},
    },
};

#[derive(Copy, Clone, Debug, Mappable)]
pub enum Slot {
    Weapon,
    Chest,
    Head,
    Feet,
}

#[derive(Component)]
pub struct ItemAssignment {
    items: EnumMap<Slot, Option<Item>>,
}

impl Default for ItemAssignment {
    fn default() -> Self {
        Self {
            items: EnumMap::new(|slot| match slot {
                Slot::Weapon => None,
                Slot::Chest => None,
                Slot::Head => None,
                Slot::Feet => None,
            }),
        }
    }
}

#[derive(Event, Deserialize, Serialize)]
pub struct OpenItemAssignment;

#[derive(Resource, Deref, DerefMut)]
pub struct ActiveBuilding(HashMap<Entity, Entity>);

pub fn open_assignment_dialog(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    mut active: ResMut<ActiveBuilding>,
    client_player_map: Res<ClientPlayerMap>,
) {
    for event in interactions.read() {
        let InteractionType::ItemAssignment = &event.interaction else {
            continue;
        };

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*client_player_map.get_network_entity(&event.player).unwrap()),
            event: OpenItemAssignment,
        });

        active.insert(event.player, event.interactable);
    }
}
