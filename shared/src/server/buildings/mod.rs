use bevy::prelude::*;

use gold_farm::{enable_goldfarm, gold_farm_output};
use item_assignment::ItemAssignmentPlugins;
use recruiting::{assign_offset, check_recruit, recruit_commander, recruit_units};
use respawn::respawn_units;
use siege_camp::siege_camp_lifetime;

use crate::{
    GameState, Owner,
    map::buildings::{BuildStatus, Building, HealthIndicator, respawn_timer},
    networking::Inventory,
    server::players::interaction::Interactable,
};

use super::players::interaction::{InteractionTriggeredEvent, InteractionType};

mod respawn;

pub mod gold_farm;
pub mod item_assignment;
pub mod recruiting;
pub mod siege_camp;

#[derive(Clone)]
pub struct BuildingEventInfo {
    pub player_entity: Entity,
    pub building_entity: Entity,
    pub building: Building,
}

#[derive(Message, Deref)]
struct BuildingChangeStart(pub BuildingEventInfo);

#[derive(Component)]
struct BuildingConstructing {
    timer: Timer,
    change: BuildingEventInfo,
}

#[derive(Message, Deref)]
pub struct BuildingChangeEnd(pub BuildingEventInfo);

pub struct BuildingsPlugins;

impl Plugin for BuildingsPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(ItemAssignmentPlugins)
            .add_message::<BuildingChangeStart>()
            .add_message::<BuildingChangeEnd>();

        app.add_observer(recruit_units);
        app.add_observer(recruit_commander);
        app.add_observer(assign_offset);
        app.add_systems(
            FixedUpdate,
            (
                gold_farm_output.run_if(in_state(GameState::GameSession)),
                (respawn_timer, respawn_units).chain(),
                siege_camp_lifetime,
                progess_construction,
                (
                    (check_recruit, check_building_interaction)
                        .run_if(on_message::<InteractionTriggeredEvent>),
                    (
                        (start_construction).run_if(on_message::<BuildingChangeStart>),
                        (end_construction, enable_goldfarm).run_if(on_message::<BuildingChangeEnd>),
                    ),
                )
                    .chain(),
            ),
        );
    }
}

fn check_building_interaction(
    mut interactions: MessageReader<InteractionTriggeredEvent>,
    mut writer: MessageWriter<BuildingChangeStart>,
    player: Query<&Inventory>,
    building: Query<(Entity, &Building, &BuildStatus)>,
) -> Result {
    for event in interactions.read() {
        let InteractionType::Building = &event.interaction else {
            continue;
        };
        info!("Checking building interact.");

        let inventory = player.get(event.player)?;

        let (entity, building, status) = building.get(event.interactable)?;

        match status {
            BuildStatus::Constructing => {
                continue;
            }
            BuildStatus::Marker => {
                if !inventory.gold.ge(&building.cost().gold) {
                    continue;
                }
                writer.write(BuildingChangeStart(BuildingEventInfo {
                    player_entity: event.player,
                    building_entity: entity,
                    building: *building,
                }));
            }
            BuildStatus::Built { indicator: _ } => {
                if building.can_upgrade() {
                    if !inventory
                        .gold
                        .ge(&building.upgrade_building().unwrap().cost().gold)
                    {
                        continue;
                    }
                    writer.write(BuildingChangeStart(BuildingEventInfo {
                        player_entity: event.player,
                        building_entity: entity,
                        building: building.upgrade_building().unwrap_or(*building),
                    }));
                }
            }
            BuildStatus::Destroyed => {
                if !inventory.gold.ge(&building.cost().gold) {
                    continue;
                }
                writer.write(BuildingChangeStart(BuildingEventInfo {
                    player_entity: event.player,
                    building_entity: entity,
                    building: *building,
                }));
            }
        }
    }
    Ok(())
}

fn start_construction(
    mut events: MessageReader<BuildingChangeStart>,
    mut inventory: Query<&mut Inventory>,
    mut building_query: Query<&mut Building>,
    mut commands: Commands,
) -> Result {
    for event in events.read() {
        let mut building_entity = commands.entity(event.building_entity);
        let building = event.building;

        let mut building_state = building_query.get_mut(event.building_entity)?;
        *building_state = building;

        info!("Start constructing: {:?}", building);

        building_entity
            .insert((
                BuildingConstructing {
                    timer: Timer::from_seconds(building.time(), TimerMode::Once),
                    change: (**event).clone(),
                },
                BuildStatus::Constructing,
            ))
            .remove::<Interactable>();

        let mut inventory = inventory.get_mut(event.player_entity)?;
        inventory.gold -= building.cost().gold;
    }
    Ok(())
}

fn progess_construction(
    mut query: Query<(Entity, &mut BuildingConstructing)>,
    time: Res<Time>,
    mut writer: MessageWriter<BuildingChangeEnd>,
    mut commands: Commands,
) {
    for (entity, mut building) in &mut query {
        building.timer.tick(time.delta());

        if building.timer.is_finished() {
            writer.write(BuildingChangeEnd(building.change.clone()));
            commands.entity(entity).remove::<BuildingConstructing>();
        }
    }
}

fn end_construction(
    mut events: MessageReader<BuildingChangeEnd>,
    owner_query: Query<&Owner>,
    mut commands: Commands,
) -> Result {
    for event in events.read() {
        let building = event.building;

        info!("End constructing: {:?}", building);

        let mut building_commands = commands.entity(event.building_entity);
        building_commands.insert((
            building.health(),
            building.collider(),
            BuildStatus::Built {
                indicator: HealthIndicator::Healthy,
            },
        ));

        let owner = owner_query.get(event.building_entity)?.entity()?;

        if building.can_upgrade() {
            building_commands.insert(Interactable {
                kind: InteractionType::Building,
                restricted_to: Some(owner),
            });
        }

        if building.is_recruit_building() {
            building_commands.insert(Interactable {
                kind: InteractionType::Recruit,
                restricted_to: Some(owner),
            });
        }
    }
    Ok(())
}
