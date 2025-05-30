use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use bevy_replicon::prelude::{FromClient, SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    ClientPlayerMap,
    enum_map::*,
    server::{
        buildings::recruiting::{FlagAssignment, FlagHolder},
        physics::attachment::AttachedTo,
        players::interaction::{InteractionTriggeredEvent, InteractionType},
    },
};

#[derive(Resource, Default, DerefMut, Deref)]
struct ActiveCommander(HashMap<Entity, Entity>);

#[derive(Event, Serialize, Deserialize)]
pub struct CommanderInteraction {
    pub commander: Entity,
}

impl MapEntities for CommanderInteraction {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.commander = entity_mapper.get_mapped(self.commander);
    }
}

#[derive(Event)]
struct SlotInteraction {
    command: SlotCommand,
    player: Entity,
    commander: Entity,
    flag: Option<Entity>,
    selected_slot: CommanderSlot,
}

#[derive(PartialEq, Eq)]
enum SlotCommand {
    Assign,
    Remove,
    Swap,
}

#[derive(Event, Serialize, Deserialize, Copy, Clone, Mappable, Eq, PartialEq)]
pub enum CommanderSlot {
    Front,
    Middle,
    Back,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SlotsAssignments {
    pub slots: EnumMap<CommanderSlot, Option<Entity>>,
}

impl MapEntities for SlotsAssignments {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.slots.iter_mut().for_each(|entity| {
            *entity = entity.take().map(|entity| entity_mapper.get_mapped(entity));
        });
    }
}

impl Default for SlotsAssignments {
    fn default() -> Self {
        Self {
            slots: EnumMap::new(|slot| match slot {
                CommanderSlot::Front => None,
                CommanderSlot::Middle => None,
                CommanderSlot::Back => None,
            }),
        }
    }
}

pub struct CommanderPlugin;

impl Plugin for CommanderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCommander>();
        app.add_event::<SlotInteraction>();

        app.add_observer(handle_slot_selection);
        app.add_observer(assign_flag_to_slot);
        app.add_observer(remove_flag_from_slot);
        app.add_observer(swap_flag_from_slot);

        app.add_systems(
            FixedUpdate,
            commander_interaction.run_if(on_event::<InteractionTriggeredEvent>),
        );
    }
}

fn commander_interaction(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    client_player_map: Res<ClientPlayerMap>,
    mut active: ResMut<ActiveCommander>,
) {
    for event in interactions.read() {
        let InteractionType::CommanderInteraction = &event.interaction else {
            continue;
        };

        let commander = event.interactable;
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*client_player_map.get_network_entity(&event.player).unwrap()),
            event: CommanderInteraction { commander },
        });

        active.insert(event.player, commander);
    }
}

fn handle_slot_selection(
    trigger: Trigger<FromClient<CommanderSlot>>,
    active: Res<ActiveCommander>,
    slots: Query<&SlotsAssignments>,
    client_player_map: ResMut<ClientPlayerMap>,
    mut commands: Commands,
    flag: Query<&FlagHolder>,
) {
    let player = client_player_map.get(&trigger.client_entity).unwrap();
    let commander = active.0.get(player).unwrap();
    let slot_assignments = slots.get(*commander).unwrap();

    let selected_slot = trigger.event;
    let is_slot_occupied = slot_assignments.slots.get(selected_slot).is_some();

    let player_flag = flag.get(*player).map(|f| f.0).ok();

    match (is_slot_occupied, player_flag.is_some()) {
        (true, true) => {
            commands.trigger(SlotInteraction {
                command: SlotCommand::Swap,
                player: player.clone(),
                commander: commander.clone(),
                flag: player_flag,
                selected_slot,
            });
        }
        (true, false) => {
            commands.trigger(SlotInteraction {
                command: SlotCommand::Remove,
                player: player.clone(),
                commander: commander.clone(),
                flag: None,
                selected_slot,
            });
        }
        (false, true) => {
            commands.trigger(SlotInteraction {
                command: SlotCommand::Assign,
                player: player.clone(),
                commander: commander.clone(),
                flag: player_flag,
                selected_slot,
            });
        }
        (false, false) => return,
    }
}

fn assign_flag_to_slot(
    trigger: Trigger<SlotInteraction>,
    mut commands: Commands,
    mut slots: Query<(&mut SlotsAssignments, &FlagAssignment)>,
) {
    let SlotCommand::Assign = trigger.command else {
        return;
    };

    let (mut slot_assignments, flag_assignment) = slots.get_mut(trigger.commander).unwrap();
    let flag = trigger.flag.unwrap();

    // Prevent assigning own flag
    if flag_assignment.0 == flag {
        return;
    }

    slot_assignments
        .slots
        .set(trigger.selected_slot, Some(flag));

    commands.entity(flag).insert(AttachedTo(trigger.commander));
    commands.entity(trigger.player).remove::<FlagHolder>();
}

fn remove_flag_from_slot(
    trigger: Trigger<SlotInteraction>,
    mut commands: Commands,
    mut slots: Query<&mut SlotsAssignments>,
) {
    let SlotCommand::Remove = trigger.command else {
        return;
    };

    let mut slot_assignments = slots.get_mut(trigger.commander).unwrap();

    let flag = slot_assignments
        .slots
        .set(trigger.selected_slot, None)
        .unwrap();

    commands.entity(flag).insert(AttachedTo(trigger.player));
    commands.entity(trigger.player).insert(FlagHolder(flag));
}

fn swap_flag_from_slot(
    trigger: Trigger<SlotInteraction>,
    mut commands: Commands,
    mut slots: Query<&mut SlotsAssignments>,
) {
    let SlotCommand::Swap = trigger.command else {
        return;
    };

    let mut slot_assignments = slots.get_mut(trigger.commander).unwrap();
    let flag = trigger.flag.unwrap();

    let slot_flag = slot_assignments
        .slots
        .set(trigger.selected_slot, Some(flag))
        .unwrap();

    commands.entity(flag).insert(AttachedTo(trigger.commander));
    commands
        .entity(slot_flag)
        .insert(AttachedTo(trigger.player));
    commands
        .entity(trigger.player)
        .insert(FlagHolder(slot_flag));
}
