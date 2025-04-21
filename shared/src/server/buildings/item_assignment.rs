use std::fmt;

use bevy::prelude::*;

use bevy::{ecs::entity::MapEntities, utils::HashMap};
use bevy_replicon::prelude::{Replicated, SendMode, ServerTriggerExt, ToClients};
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
        app.init_resource::<ActiveBuilding>().add_systems(
            Update,
            start_assignment_dialog.run_if(on_event::<InteractionTriggeredEvent>),
        );
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
#[require(Replicated)]
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

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ActiveBuilding(HashMap<Entity, Entity>);

pub fn start_assignment_dialog(
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
