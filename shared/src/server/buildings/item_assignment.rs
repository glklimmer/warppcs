use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use bevy::ecs::entity::MapEntities;
use bevy_replicon::prelude::{FromClient, Replicated, SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    ClientPlayerMap, ClientPlayerMapExt, Player,
    enum_map::*,
    map::buildings::{Building, BuildingType, RespawnZone},
    networking::Inventory,
    server::players::{
        interaction::{InteractionTriggeredEvent, InteractionType},
        items::{Item, ItemType},
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
                start_assignment_dialog.run_if(on_message::<InteractionTriggeredEvent>),
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
pub struct CloseBuildingDialog(usize);

#[derive(Event, Deserialize, Serialize, Deref)]
pub struct AssignItem(Item);

impl AssignItem {
    pub fn new(item: Item) -> Self {
        Self(item)
    }
}

#[derive(Event, Deserialize, Serialize)]
pub struct StartBuild(pub usize);

#[derive(Resource, Default, Deref, DerefMut)]
struct ActiveBuilding(HashMap<Entity, Entity>);

trait ActiveBuildingExt {
    fn get_entity(&self, entity: &Entity) -> Result<&Entity>;
}

impl ActiveBuildingExt for ActiveBuilding {
    fn get_entity(&self, entity: &Entity) -> Result<&Entity> {
        self.get(entity)
            .ok_or("No building set as ActiveBuilding".into())
    }
}

fn check_start_building(
    trigger: On<FromClient<StartBuild>>,
    mut interactions: MessageWriter<InteractionTriggeredEvent>,
    assignment: Query<&ItemAssignment>,
    active: Res<ActiveBuilding>,
    client_player_map: Res<ClientPlayerMap>,
    players: Query<&Player>,
    mut commands: Commands,
) -> Result {
    let player_entity = *client_player_map.get_player(&trigger.client_id)?;
    let player = players.get(player_entity)?;
    let active_building = *active.get_entity(&player_entity)?;

    let assignment = assignment.get(active_building)?;
    let items: Vec<_> = match assignment
        .items
        .clone()
        .into_iter()
        .collect::<Option<Vec<Item>>>()
    {
        Some(v) => v,
        None => {
            info!("Not all items assigned!");
            return Ok(());
        }
    };

    if let Some(weapon) = items.into_iter().find_map(|item| {
        if let ItemType::Weapon(weapon) = item.item_type {
            Some(weapon.unit_type())
        } else {
            None
        }
    }) {
        commands.entity(active_building).insert((
            Building {
                building_type: BuildingType::Unit { weapon },
                color: player.color,
            },
            RespawnZone::default(),
        ));
    } else {
        info!("No weapon in assigned items!");
    }

    interactions.write(InteractionTriggeredEvent {
        player: player_entity,
        interactable: active_building,
        interaction: InteractionType::Building,
    });

    commands.server_trigger(ToClients {
        mode: SendMode::Direct(trigger.client_id),
        message: CloseBuildingDialog(0),
    });
    Ok(())
}

fn start_assignment_dialog(
    mut interactions: MessageReader<InteractionTriggeredEvent>,
    mut active: ResMut<ActiveBuilding>,
    client_player_map: Res<ClientPlayerMap>,
    mut commands: Commands,
) -> Result {
    for event in interactions.read() {
        let InteractionType::ItemAssignment = &event.interaction else {
            continue;
        };

        let player = client_player_map.get_network_entity(&event.player)?;

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*player),
            message: OpenBuildingDialog {
                building: event.interactable,
            },
        });

        active.insert(event.player, event.interactable);
    }
    Ok(())
}

fn assign_item(
    trigger: On<FromClient<AssignItem>>,
    active: Res<ActiveBuilding>,
    mut assignment: Query<&mut ItemAssignment>,
    mut inventory: Query<&mut Inventory>,
    client_player_map: Res<ClientPlayerMap>,
) -> Result {
    let player = client_player_map.get_player(&trigger.client_id)?;
    let active_building = *active.get_entity(player)?;
    let mut inventory = inventory.get_mut(*player)?;

    let item = &***trigger;
    let Some(index) = inventory.items.iter().position(|inv_item| inv_item == item) else {
        return Ok(());
    };
    inventory.items.remove(index);

    let mut assignment = assignment.get_mut(active_building)?;
    let maybe_item = assignment.items.set(item.slot(), Some(item.clone()));
    if let Some(item) = maybe_item {
        inventory.items.push(item);
    }
    Ok(())
}
