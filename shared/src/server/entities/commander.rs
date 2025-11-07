use bevy::prelude::*;

use bevy::{ecs::entity::MapEntities, platform::collections::HashMap};
use bevy_replicon::prelude::{FromClient, SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::ClientPlayerMapExt;
use crate::networking::UnitType;
use crate::server::buildings::recruiting::{Flag, FlagAssignment};
use crate::server::game_scenes::GameSceneId;
use crate::{
    ClientPlayerMap, Owner, Vec3LayerExt,
    enum_map::*,
    map::Layers,
    server::{
        buildings::{recruiting::FlagHolder, siege_camp::SiegeCamp},
        physics::attachment::AttachedTo,
        players::interaction::{Interactable, InteractionTriggeredEvent, InteractionType},
    },
};

#[derive(Resource, Default, DerefMut, Deref)]
struct ActiveCommander(HashMap<Entity, Entity>);

trait ActiveCommanderExt {
    fn get_entity(&self, entity: &Entity) -> Result<&Entity>;
}

impl ActiveCommanderExt for ActiveCommander {
    fn get_entity(&self, entity: &Entity) -> Result<&Entity> {
        self.get(entity)
            .ok_or("No commander set as ActiveCommander".into())
    }
}

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
    selected_slot: ArmyPosition,
}

#[derive(PartialEq, Eq)]
enum SlotCommand {
    Assign,
    Remove,
    Swap,
}

#[derive(Event, Serialize, Deserialize, Copy, Clone, Mappable, PartialEq, Eq, Debug)]
pub enum ArmyPosition {
    Front,
    Middle,
    Back,
}

#[derive(Event, Serialize, Deserialize)]
pub struct CommanderCampInteraction;

#[derive(Event, Serialize, Deserialize)]
pub struct CommanderPickFlag;

#[derive(Event, Serialize, Deserialize)]
pub struct CommanderAssignmentRequest;

#[derive(Event, Serialize, Deserialize)]
pub struct CommanderAssignmentReject;

#[derive(Event, Serialize, Deserialize)]
pub struct Assignment {
    player: Entity,
    slot: ArmyPosition,
}

pub const BASE_FORMATION_WIDTH: f32 = 50.;
pub const BASE_FORMATION_OFFSET: f32 = 5.;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ArmyFlagAssignments {
    #[entities]
    pub flags: EnumMap<ArmyPosition, Option<Entity>>,
}

impl Default for ArmyFlagAssignments {
    fn default() -> Self {
        Self {
            flags: EnumMap::new(|slot| match slot {
                ArmyPosition::Front => None,
                ArmyPosition::Middle => None,
                ArmyPosition::Back => None,
            }),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ArmyFormation {
    #[entities]
    pub positions: EnumMap<ArmyPosition, Entity>,
}

pub struct CommanderPlugin;

impl Plugin for CommanderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCommander>()
            .add_event::<SlotInteraction>()
            .add_event::<CommanderCampInteraction>()
            .add_event::<Assignment>()
            .add_observer(commander_assignment_validation)
            .add_observer(handle_slot_selection)
            .add_observer(handle_camp_interaction)
            .add_observer(assign_flag_to_formation)
            .add_observer(remove_flag_from_formation)
            .add_observer(swap_flag_from_formation)
            .add_observer(handle_pick_flag)
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
) -> Result {
    for event in interactions.read() {
        let InteractionType::Commander = &event.interaction else {
            continue;
        };

        let player = client_player_map.get_network_entity(&event.player)?;

        let commander = event.interactable;
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*player),
            event: CommanderInteraction { commander },
        });

        active.insert(event.player, commander);
    }
    Ok(())
}

fn handle_pick_flag(
    trigger: Trigger<FromClient<CommanderPickFlag>>,
    active: Res<ActiveCommander>,
    client_player_map: ResMut<ClientPlayerMap>,
    formations: Query<&ArmyFlagAssignments>,
    commander_flag_assignment: Query<&FlagAssignment>,
    flag_holder: Query<Option<&FlagHolder>>,
    flag: Query<(Entity, &Flag)>,
    mut commands: Commands,
) -> Result {
    let player = client_player_map.get_player(&trigger.client_entity)?;
    let commander = active.get_entity(player)?;
    let commander_flag = commander_flag_assignment.get(*commander)?;
    let army_flag_assignments = formations.get(*commander)?;
    let (flag_entity, _) = flag.get(**commander_flag)?;

    if let Ok(Some(current_flag)) = flag_holder.get(*player) {
        let all_army_flags_assigned = army_flag_assignments.flags.iter().all(Option::is_some);
        let unit_type = flag.get(**current_flag)?.1.unit_type;

        if all_army_flags_assigned || unit_type.eq(&UnitType::Commander) {
            commands.entity(**current_flag).remove::<AttachedTo>();
        } else {
            // Assign player flag to any empty formation slots
            army_flag_assignments
                .flags
                .iter_enums()
                .filter(|(_, flag)| flag.is_none())
                .for_each(|(formation, _)| {
                    commands.trigger(Assignment {
                        player: *player,
                        slot: formation,
                    });
                });
        }
    }

    commands.entity(*player).insert(FlagHolder(flag_entity));
    commands.entity(flag_entity).insert(AttachedTo(*player));
    Ok(())
}

fn handle_camp_interaction(
    trigger: Trigger<FromClient<CommanderCampInteraction>>,
    active: Res<ActiveCommander>,
    client_player_map: ResMut<ClientPlayerMap>,
    query: Query<(&Transform, &GameSceneId)>,
    mut commands: Commands,
) -> Result {
    let player = client_player_map.get_player(&trigger.client_entity)?;
    let commander = active.get_entity(player)?;
    let (commander_transform, game_scene_id) = query.get(*commander)?;
    let commander_pos = commander_transform.translation;

    commands.spawn((
        SiegeCamp::default(),
        commander_pos.with_layer(Layers::Building),
        Owner::Player(*player),
        *game_scene_id,
    ));
    Ok(())
}

fn commander_assignment_validation(
    trigger: Trigger<FromClient<ArmyPosition>>,
    client_player_map: ResMut<ClientPlayerMap>,
    flag_holder: Query<&FlagHolder>,
    flag: Query<&Flag>,
    mut commands: Commands,
) -> Result {
    let player = client_player_map.get_player(&trigger.client_entity)?;
    let player_flag = flag_holder.get(*player);

    if let Ok(unit_flag) = player_flag {
        let unit = flag.get(**unit_flag)?;
        if let UnitType::Commander = unit.unit_type {
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(trigger.client_entity),
                event: CommanderAssignmentReject,
            });
            return Ok(());
        }
    };

    commands.trigger(Assignment {
        player: *player,
        slot: trigger.event,
    });
    Ok(())
}

fn handle_slot_selection(
    trigger: Trigger<Assignment>,
    active: Res<ActiveCommander>,
    formations: Query<&ArmyFlagAssignments>,
    mut commands: Commands,
    flag_holder: Query<&FlagHolder>,
) -> Result {
    let player = &trigger.player;
    let commander = active.get_entity(player)?;
    let formation = formations.get(*commander)?;

    let selected_slot = trigger.slot;
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
    Ok(())
}

fn assign_flag_to_formation(
    trigger: Trigger<SlotInteraction>,
    mut commanders: Query<(&mut ArmyFlagAssignments, &ArmyFormation)>,
    mut commands: Commands,
) -> Result {
    let SlotCommand::Assign = trigger.command else {
        return Ok(());
    };

    let (mut units_assignments, army_formation) = commanders.get_mut(trigger.commander)?;
    let flag = trigger.flag.ok_or("No new flag provided")?;

    units_assignments
        .flags
        .set(trigger.selected_slot, Some(flag));

    let formation = army_formation.positions.get(trigger.selected_slot);

    commands
        .entity(flag)
        .insert(AttachedTo(*formation))
        .remove::<Interactable>();
    commands.entity(trigger.player).remove::<FlagHolder>();
    Ok(())
}

fn remove_flag_from_formation(
    trigger: Trigger<SlotInteraction>,
    mut commanders: Query<&mut ArmyFlagAssignments>,
    mut commands: Commands,
) -> Result {
    let SlotCommand::Remove = trigger.command else {
        return Ok(());
    };

    let mut units_assignments = commanders.get_mut(trigger.commander)?;

    let flag = units_assignments
        .flags
        .set(trigger.selected_slot, None)
        .ok_or("There is no flag to be removed")?;

    commands.entity(flag).insert((
        AttachedTo(trigger.player),
        Interactable {
            kind: InteractionType::Flag,
            restricted_to: Some(trigger.player),
        },
    ));

    commands.entity(trigger.player).insert(FlagHolder(flag));
    Ok(())
}

fn swap_flag_from_formation(
    trigger: Trigger<SlotInteraction>,
    mut commanders: Query<(&mut ArmyFlagAssignments, &ArmyFormation)>,
    mut commands: Commands,
) -> Result {
    let SlotCommand::Swap = trigger.command else {
        return Ok(());
    };

    let (mut army_flag_assignments, army_formation) = commanders.get_mut(trigger.commander)?;

    let new_flag = trigger.flag.ok_or("No new flag provided")?;

    let old_flag = army_flag_assignments
        .flags
        .get(trigger.selected_slot)
        .ok_or("There is no flag to be swapped")?;

    army_flag_assignments
        .flags
        .set(trigger.selected_slot, Some(new_flag));

    let formation = army_formation.positions.get(trigger.selected_slot);

    commands
        .entity(new_flag)
        .insert(AttachedTo(*formation))
        .remove::<Interactable>();

    commands.entity(old_flag).insert((
        AttachedTo(trigger.player),
        Interactable {
            kind: InteractionType::Flag,
            restricted_to: Some(trigger.player),
        },
    ));
    commands.entity(trigger.player).insert(FlagHolder(old_flag));
    Ok(())
}
