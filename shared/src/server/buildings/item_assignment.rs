use bevy::{ecs::entity::MapEntities, prelude::*};

use bevy::utils::HashMap;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, SendMode, ServerTriggerAppExt, ServerTriggerExt, ToClients,
};
use serde::{Deserialize, Serialize};

use crate::{
    ClientPlayerMap,
    enum_map::*,
    server::players::{
        interaction::{InteractionTriggeredEvent, InteractionType},
        items::Item,
    },
};

pub struct ItemAssignmentPlugins;

impl Plugin for ItemAssignmentPlugins {
    fn build(&self, app: &mut App) {
        app.replicate::<ItemAssignment>()
            .add_mapped_server_trigger::<OpenItemAssignment>(Channel::Ordered);
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Mappable)]
pub enum Slot {
    Weapon,
    Chest,
    Head,
    Feet,
}

#[derive(Component, Serialize, Deserialize)]
pub struct ItemAssignment {
    pub items: EnumMap<Slot, Option<Item>>,
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
pub struct OpenItemAssignment {
    pub building: Entity,
}

impl MapEntities for OpenItemAssignment {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.building = entity_mapper.map_entity(self.building);
    }
}

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
            event: OpenItemAssignment {
                building: event.interactable,
            },
        });

        active.insert(event.player, event.interactable);
    }
}
