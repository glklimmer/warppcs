use bevy::prelude::*;

use bevy::{ecs::entity::MapEntities, platform::collections::HashMap};
use bevy_replicon::prelude::{FromClient, SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    ClientPlayerMap, Owner, Vec3LayerExt,
    enum_map::*,
    map::Layers,
    server::{
        buildings::{
            recruiting::{FlagAssignment, FlagHolder},
            siege_camp::SiegeCamp,
        },
        physics::attachment::AttachedTo,
        players::interaction::{Interactable, InteractionTriggeredEvent, InteractionType},
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
    selected_slot: CommanderFormation,
}

#[derive(PartialEq, Eq)]
enum SlotCommand {
    Assign,
    Remove,
    Swap,
}

#[derive(Event, Serialize, Deserialize, Copy, Clone, Mappable, PartialEq, Eq, Debug)]
pub enum CommanderFormation {
    Front,
    Middle,
    Back,
}

#[derive(Event, Serialize, Deserialize, Copy, Clone)]
pub struct CommanderCampInteraction;

pub const BASE_FORMATION_WIDTH: f32 = 50.;
pub const BASE_FORMATION_OFFSET: f32 = 5.;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ArmyFlagAssignments {
    #[entities]
    pub flags: EnumMap<CommanderFormation, Option<Entity>>,
}

impl Default for ArmyFlagAssignments {
    fn default() -> Self {
        Self {
            flags: EnumMap::new(|slot| match slot {
                CommanderFormation::Front => None,
                CommanderFormation::Middle => None,
                CommanderFormation::Back => None,
            }),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ArmyFormation {
    #[entities]
    pub positions: EnumMap<CommanderFormation, Entity>,
}

pub struct CommanderPlugin;

impl Plugin for CommanderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCommander>()
            .add_event::<SlotInteraction>()
            .add_event::<CommanderCampInteraction>()
            .add_observer(handle_slot_selection)
            .add_observer(handle_camp_interaction)
            .add_observer(assign_flag_to_formation)
            .add_observer(remove_flag_from_formation)
            .add_observer(swap_flag_from_formation)
            .add_systems(
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
        let InteractionType::Commander = &event.interaction else {
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

fn handle_camp_interaction(
    trigger: Trigger<FromClient<CommanderCampInteraction>>,
    active: Res<ActiveCommander>,
    client_player_map: ResMut<ClientPlayerMap>,
    mut commands: Commands,
    transform: Query<&Transform>,
) {
    let player = client_player_map.get(&trigger.client_entity).unwrap();
    let commander = active.0.get(player).unwrap();
    let commander_transform = transform.get(*commander).unwrap();
    let commander_pos = commander_transform.translation;

    commands.spawn((
        SiegeCamp::default(),
        commander_pos.with_layer(Layers::Building),
        Owner::Player(*player),
    ));
}

fn handle_slot_selection(
    trigger: Trigger<FromClient<CommanderFormation>>,
    active: Res<ActiveCommander>,
    formations: Query<&ArmyFlagAssignments>,
    client_player_map: ResMut<ClientPlayerMap>,
    mut commands: Commands,
    flag_holder: Query<&FlagHolder>,
) {
    let player = client_player_map.get(&trigger.client_entity).unwrap();
    let commander = active.0.get(player).unwrap();
    let formation = formations.get(*commander).unwrap();

    let selected_slot = trigger.event;
    let is_slot_occupied = formation.flags.get(selected_slot).is_some();

    let player_flag = flag_holder.get(*player).map(|flag| **flag).ok();

    match (is_slot_occupied, player_flag.is_some()) {
        (true, true) => {
            commands.trigger(SlotInteraction {
                command: SlotCommand::Swap,
                player: *player,
                commander: *commander,
                flag: player_flag,
                selected_slot,
            });
        }
        (true, false) => {
            commands.trigger(SlotInteraction {
                command: SlotCommand::Remove,
                player: *player,
                commander: *commander,
                flag: None,
                selected_slot,
            });
        }
        (false, true) => {
            commands.trigger(SlotInteraction {
                command: SlotCommand::Assign,
                player: *player,
                commander: *commander,
                flag: player_flag,
                selected_slot,
            });
        }
        (false, false) => {}
    }
}

fn assign_flag_to_formation(
    trigger: Trigger<SlotInteraction>,
    mut commands: Commands,
    mut commanders: Query<(&mut ArmyFlagAssignments, &ArmyFormation, &FlagAssignment)>,
) {
    let SlotCommand::Assign = trigger.command else {
        return;
    };

    let (mut units_assignments, army_formation, flag_assignment) =
        commanders.get_mut(trigger.commander).unwrap();
    let flag = trigger.flag.unwrap();

    // Prevent assigning commander flag to formation
    if **flag_assignment == flag {
        return;
    }

    units_assignments
        .flags
        .set(trigger.selected_slot, Some(flag));

    let formation = army_formation.positions.get(trigger.selected_slot);

    commands
        .entity(flag)
        .insert(AttachedTo(*formation))
        .remove::<Interactable>();
    commands.entity(trigger.player).remove::<FlagHolder>();
}

fn remove_flag_from_formation(
    trigger: Trigger<SlotInteraction>,
    mut commands: Commands,
    mut commanders: Query<&mut ArmyFlagAssignments>,
) {
    let SlotCommand::Remove = trigger.command else {
        return;
    };

    let mut units_assignments = commanders.get_mut(trigger.commander).unwrap();

    let flag = units_assignments
        .flags
        .set(trigger.selected_slot, None)
        .unwrap();

    commands.entity(flag).insert((
        AttachedTo(trigger.player),
        Interactable {
            kind: InteractionType::Flag,
            restricted_to: Some(trigger.player),
        },
    ));

    commands.entity(trigger.player).insert(FlagHolder(flag));
}

fn swap_flag_from_formation(
    trigger: Trigger<SlotInteraction>,
    mut commands: Commands,
    mut commanders: Query<(&mut ArmyFlagAssignments, &ArmyFormation)>,
) {
    let SlotCommand::Swap = trigger.command else {
        return;
    };

    let (mut army_flag_assignments, army_formation) =
        commanders.get_mut(trigger.commander).unwrap();

    let new_flag = trigger.flag.unwrap();
    let old_flag = army_flag_assignments
        .flags
        .get(trigger.selected_slot)
        .unwrap();

    army_flag_assignments
        .flags
        .set(trigger.selected_slot, Some(new_flag))
        .unwrap();

    let formation = army_formation.positions.get(trigger.selected_slot);

    commands
        .entity(new_flag)
        .insert((AttachedTo(*formation), Visibility::Hidden))
        .remove::<Interactable>();

    commands.entity(old_flag).insert((
        AttachedTo(trigger.player),
        Visibility::Visible,
        Interactable {
            kind: InteractionType::Flag,
            restricted_to: Some(trigger.player),
        },
    ));
    commands.entity(trigger.player).insert(FlagHolder(old_flag));
}
