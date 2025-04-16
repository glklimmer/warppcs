use bevy::{ecs::entity::MapEntities, prelude::*, utils::HashMap};
use bevy_replicon::prelude::{FromClient, SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    ClientPlayerMap,
    server::{
        buildings::recruiting::FlagHolder,
        physics::attachment::AttachedTo,
        players::interaction::{InteractionTriggeredEvent, InteractionType},
    },
};

#[derive(Resource, Default)]
struct ActiveCommander(HashMap<Entity, Entity>);

#[derive(Event, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum CommanderInteraction {
    Options(Entity),
    Map,
    AssignementSlots,
    AssigneToSlot,
}

#[derive(Event, Serialize, Deserialize, Copy, Clone)]
pub enum SlotSelection {
    Front,
    Middle,
    Back,
}

#[derive(Component, Serialize, Deserialize)]
pub struct SlotsAssignments {
    pub front: Option<Entity>,
    pub middle: Option<Entity>,
    pub back: Option<Entity>,
}

impl Default for SlotsAssignments {
    fn default() -> Self {
        Self {
            front: None,
            middle: None,
            back: None,
        }
    }
}

impl MapEntities for SlotsAssignments {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.back = Some(entity_mapper.map_entity(self.back.unwrap()));
        self.middle = Some(entity_mapper.map_entity(self.middle.unwrap()));
        self.front = Some(entity_mapper.map_entity(self.front.unwrap()));
    }
}

impl MapEntities for CommanderInteraction {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        let CommanderInteraction::Options(entity) = *self else {
            return;
        };
        *self = CommanderInteraction::Options(entity_mapper.map_entity(entity));
    }
}

pub struct CommanderPlugin;

impl Plugin for CommanderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCommander>();
        app.add_observer(get_slot_interaction);
        app.add_systems(
            FixedUpdate,
            test.run_if(on_event::<InteractionTriggeredEvent>),
        );
    }
}

fn test(
    mut commands: Commands,
    mut interactions: EventReader<InteractionTriggeredEvent>,
    client_player_map: Res<ClientPlayerMap>,
    mut active: ResMut<ActiveCommander>,
) {
    for event in interactions.read() {
        let InteractionType::CommanderInteraction = &event.interaction else {
            continue;
        };
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*client_player_map.get_network_entity(&event.player).unwrap()),
            event: CommanderInteraction::Options(event.interactable),
        });
        active.0.insert(event.player, event.interactable);
        println!("Got Commander")
    }
}

fn get_slot_interaction(
    trigger: Trigger<FromClient<SlotSelection>>,
    mut commands: Commands,
    mut slot: Query<(&mut SlotsAssignments)>,
    client_player_map: ResMut<ClientPlayerMap>,
    mut active: ResMut<ActiveCommander>,
    flag: Query<&FlagHolder>,
) {
    let selected_slot = trigger.event().event;
    let player = client_player_map.get(&trigger.client_entity).unwrap();
    let commander = active.0.get(player).unwrap();
    let flag = flag.get(*player).unwrap();

    let mut slot_assignments = slot.get_single_mut().unwrap();
    match selected_slot {
        SlotSelection::Front => slot_assignments.front = Some(flag.0),
        SlotSelection::Middle => slot_assignments.middle = Some(flag.0),
        SlotSelection::Back => slot_assignments.back = Some(flag.0),
    };

    commands.entity(flag.0).insert(AttachedTo(*commander));
    commands.entity(*player).remove::<FlagHolder>();
}
