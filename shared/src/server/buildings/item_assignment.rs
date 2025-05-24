use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use bevy::ecs::entity::MapEntities;
use bevy_replicon::prelude::{FromClient, Replicated, SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::map::buildings::Building;
use crate::networking::Inventory;
use crate::server::players::items::ItemType;
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
        app.add_observer(assign_item)
            .add_observer(check_start_building)
            .init_resource::<ActiveBuilding>()
            .add_systems(
                Update,
                start_assignment_dialog.run_if(on_event::<InteractionTriggeredEvent>),
            );
    }
}

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize, Mappable, Eq, PartialEq)]
pub enum ItemSlot {
    Weapon,
    Chest,
    Head,
    Feet,
}

#[derive(Component, Serialize, Deserialize, Clone)]
#[require(Replicated)]
pub struct ItemAssignment {
    pub items: EnumMap<ItemSlot, Option<Item>>,
}

impl Default for ItemAssignment {
    fn default() -> Self {
        Self {
            items: EnumMap::new(|slot| match slot {
                ItemSlot::Weapon => None,
                ItemSlot::Chest => None,
                ItemSlot::Head => None,
                ItemSlot::Feet => None,
            }),
        }
    }
}

#[derive(Event, Deserialize, Serialize)]
pub struct OpenBuildingDialog {
    pub building: Entity,
}

impl MapEntities for OpenBuildingDialog {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.building = entity_mapper.get_mapped(self.building);
    }
}

#[derive(Event, Deserialize, Serialize)]
pub struct CloseBuildingDialog;

#[derive(Event, Deserialize, Serialize, Deref)]
pub struct AssignItem(Item);

impl AssignItem {
    pub fn new(item: Item) -> Self {
        Self(item)
    }
}

#[derive(Event, Deserialize, Serialize)]
pub struct StartBuild;

#[derive(Resource, Default, Deref, DerefMut)]
struct ActiveBuilding(HashMap<Entity, Entity>);

fn check_start_building(
    trigger: Trigger<FromClient<StartBuild>>,
    mut commands: Commands,
    mut interactions: EventWriter<InteractionTriggeredEvent>,
    assignment: Query<&ItemAssignment>,
    active: Res<ActiveBuilding>,
    client_player_map: Res<ClientPlayerMap>,
) {
    let player = *client_player_map.get(&trigger.client_entity).unwrap();

    let active_building = *active.get(&player).unwrap();

    let assignment = assignment.get(active_building).unwrap();
    let items: Vec<_> = match assignment
        .items
        .clone()
        .into_iter()
        .collect::<Option<Vec<Item>>>()
    {
        Some(v) => v,
        None => {
            info!("Not all items assigned!");
            return;
        }
    };

    if let Some(weapon) = items.into_iter().find_map(|item| {
        if let ItemType::Weapon(weapon) = item.item_type {
            Some(weapon.unit_type())
        } else {
            None
        }
    }) {
        commands
            .entity(active_building)
            .insert(Building::Unit { weapon });
    } else {
        info!("No weapon in assigned items!");
    }

    interactions.write(InteractionTriggeredEvent {
        player,
        interactable: active_building,
        interaction: InteractionType::Building,
    });

    commands.server_trigger(ToClients {
        mode: SendMode::Direct(trigger.client_entity),
        event: CloseBuildingDialog,
    });
}

fn start_assignment_dialog(
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
            event: OpenBuildingDialog {
                building: event.interactable,
            },
        });

        active.insert(event.player, event.interactable);
    }
}

fn assign_item(
    trigger: Trigger<FromClient<AssignItem>>,
    active: Res<ActiveBuilding>,
    mut assignment: Query<&mut ItemAssignment>,
    mut inventory: Query<&mut Inventory>,
    client_player_map: Res<ClientPlayerMap>,
) {
    let player = *client_player_map.get(&trigger.client_entity).unwrap();

    let active_building = active.get(&player);
    let mut inventory = inventory.get_mut(player).unwrap();

    let item = &***trigger;
    let Some(index) = inventory.items.iter().position(|inv_item| inv_item == item) else {
        return;
    };
    inventory.items.remove(index);

    let mut assignment = assignment.get_mut(*active_building.unwrap()).unwrap();
    let maybe_item = assignment.items.set(item.slot(), Some(item.clone()));
    if let Some(item) = maybe_item {
        inventory.items.push(item);
    }
}
