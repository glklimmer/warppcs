use bevy::prelude::*;

use bevy_replicon::prelude::AppRuleExt;
use gold_farm::{enable_goldfarm, gold_farm_output};
use recruiting::{check_recruit, recruit, RecruitEvent};

use crate::{
    map::buildings::{BuildStatus, Building, RecruitBuilding},
    networking::Inventory,
    server::players::interaction::Interactable,
    Owner,
};

use super::players::interaction::{InteractionTriggeredEvent, InteractionType};

mod gold_farm;

pub mod recruiting;

pub struct CommonBuildingInfo {
    pub player_entity: Entity,
    pub entity: Entity,
    pub building_type: Building,
}

#[derive(Event)]
struct BuildingConstruction(pub CommonBuildingInfo);

#[derive(Event)]
pub struct BuildingUpgrade(pub CommonBuildingInfo);

pub struct BuildingsPlugins;

impl Plugin for BuildingsPlugins {
    fn build(&self, app: &mut App) {
        app.add_event::<RecruitEvent>();
        app.add_event::<BuildingConstruction>();
        app.add_event::<BuildingUpgrade>();

        app.add_systems(
            FixedUpdate,
            (
                (check_recruit, check_building_interaction)
                    .run_if(on_event::<InteractionTriggeredEvent>),
                (
                    (construct_building, enable_goldfarm).run_if(on_event::<BuildingConstruction>),
                    (upgrade_building,).run_if(on_event::<BuildingUpgrade>),
                    recruit.run_if(on_event::<RecruitEvent>),
                ),
            )
                .chain(),
        );

        app.add_systems(FixedUpdate, gold_farm_output);
    }
}

fn check_building_interaction(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut build: EventWriter<BuildingConstruction>,
    mut upgrade: EventWriter<BuildingUpgrade>,
    player: Query<&Inventory>,
    building: Query<(Entity, &Building, &BuildStatus)>,
) {
    for event in interactions.read() {
        let InteractionType::Building = &event.interaction else {
            continue;
        };
        info!("Checking building interact.");

        let inventory = player.get(event.player).unwrap();

        let (entity, building, status) = building.get(event.interactable).unwrap();

        let info = CommonBuildingInfo {
            player_entity: event.player,
            entity,
            building_type: *building,
        };

        match status {
            BuildStatus::Marker => {
                if !inventory.gold.ge(&building.cost().gold) {
                    continue;
                }
                build.send(BuildingConstruction(info));
            }
            BuildStatus::Built => {
                if building.can_upgrade() {
                    if !inventory
                        .gold
                        .ge(&building.upgrade_building().unwrap().cost().gold)
                    {
                        continue;
                    }
                    upgrade.send(BuildingUpgrade(info));
                }
            }
            BuildStatus::Destroyed => {
                build.send(BuildingConstruction(info));
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn construct_building(
    mut commands: Commands,
    mut builds: EventReader<BuildingConstruction>,
    building_query: Query<(&Owner, Option<&RecruitBuilding>)>,
    mut inventory: Query<&mut Inventory>,
) {
    for build in builds.read() {
        let mut building_entity = commands.entity(build.0.entity);
        let building = &build.0.building_type;

        info!("Constructing building: {:?}", building);

        building_entity.insert((building.health(), building.collider(), BuildStatus::Built));

        if !building.can_upgrade() {
            building_entity.remove::<Interactable>();
        }

        let (owner, maybe_recruit) = building_query.get(build.0.entity).unwrap();

        if maybe_recruit.is_some() {
            building_entity.insert(Interactable {
                kind: InteractionType::Recruit,
                restricted_to: Some(*owner),
            });
        }

        let mut inventory = inventory.get_mut(build.0.player_entity).unwrap();
        inventory.gold -= building.cost().gold;
    }
}

fn upgrade_building(
    mut commands: Commands,
    mut upgrade: EventReader<BuildingUpgrade>,
    mut building: Query<&mut Building>,
    mut inventory: Query<&mut Inventory>,
) {
    for upgrade in upgrade.read() {
        let mut building = building.get_mut(upgrade.0.entity).unwrap();

        let upgraded_building = &upgrade
            .0
            .building_type
            .upgrade_building()
            .expect("No Upgrade specified.");

        println!("Upgraded building: {:?}", upgraded_building);

        *building = *upgraded_building;

        commands
            .entity(upgrade.0.entity)
            .insert(upgraded_building.health())
            .insert(upgraded_building.collider());

        let mut inventory = inventory.get_mut(upgrade.0.player_entity).unwrap();
        inventory.gold -= upgraded_building.cost().gold;
    }
}
