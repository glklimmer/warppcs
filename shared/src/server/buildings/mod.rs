use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use bevy_renet::renet::ClientId;
use gold_farm::{enable_goldfarm, gold_farm_output};
use recruiting::{check_recruit, recruit, RecruitEvent};

use crate::{
    map::{
        buildings::{BuildStatus, Building, Cost, MainBuildingLevels, WallLevels},
        scenes::SceneBuildingIndicator,
        GameSceneId,
    },
    networking::{BuildingUpdate, Faction, Inventory, Owner, ServerMessages, UpdateType},
    BoxCollider,
};

use super::{
    entities::health::Health,
    networking::{SendServerMessage, ServerLobby},
    players::InteractEvent,
};

mod gold_farm;

pub mod recruiting;

pub struct CommonBuildingInfo {
    pub client_id: ClientId,
    pub player_entity: Entity,
    pub scene_id: GameSceneId,
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
                (check_recruit, check_building_interaction).run_if(on_event::<InteractEvent>),
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

pub fn building_health(building_type: &Building) -> Health {
    let hitpoints = match building_type {
        Building::MainBuilding { level } => match level {
            MainBuildingLevels::Tent => 1200.,
            MainBuildingLevels::Hall => 3600.,
            MainBuildingLevels::Castle => 6400.,
        },
        Building::Archer => 800.,
        Building::Warrior => 800.,
        Building::Pikeman => 800.,
        Building::Wall { level } => match level {
            WallLevels::Basic => 600.,
            WallLevels::Wood => 1200.,
            WallLevels::Tower => 2400.,
        },
        Building::Tower => 400.,
        Building::GoldFarm => 600.,
    };
    Health { hitpoints }
}

pub fn building_collider(building_type: &Building) -> BoxCollider {
    match building_type {
        Building::MainBuilding { level } => match level {
            MainBuildingLevels::Tent => BoxCollider {
                dimension: Vec2::new(200., 100.),
                offset: None,
            },
            MainBuildingLevels::Hall => BoxCollider {
                dimension: Vec2::new(200., 100.),
                offset: None,
            },
            MainBuildingLevels::Castle => BoxCollider {
                dimension: Vec2::new(200., 100.),
                offset: None,
            },
        },
        Building::Archer => BoxCollider {
            dimension: Vec2::new(200., 100.),
            offset: None,
        },
        Building::Warrior => BoxCollider {
            dimension: Vec2::new(200., 100.),
            offset: None,
        },
        Building::Pikeman => BoxCollider {
            dimension: Vec2::new(200., 100.),
            offset: None,
        },
        Building::Wall { level } => match level {
            WallLevels::Basic => BoxCollider {
                dimension: Vec2::new(50., 30.),
                offset: Some(Vec2::new(0., -130.)),
            },
            WallLevels::Wood => BoxCollider {
                dimension: Vec2::new(60., 100.),
                offset: Some(Vec2::new(0., -95.)),
            },
            WallLevels::Tower => BoxCollider {
                dimension: Vec2::new(110., 190.),
                offset: Some(Vec2::new(0., -45.)),
            },
        },
        Building::Tower => BoxCollider {
            dimension: Vec2::new(200., 100.),
            offset: None,
        },
        Building::GoldFarm => BoxCollider {
            dimension: Vec2::new(200., 100.),
            offset: None,
        },
    }
}

pub fn construction_cost(building_type: &Building) -> Cost {
    let gold = match building_type {
        Building::MainBuilding { level } => match level {
            MainBuildingLevels::Tent => 0,
            MainBuildingLevels::Hall => 1000,
            MainBuildingLevels::Castle => 4000,
        },
        Building::Archer => 200,
        Building::Warrior => 200,
        Building::Pikeman => 200,
        Building::Wall { level } => match level {
            WallLevels::Basic => 100,
            WallLevels::Wood => 300,
            WallLevels::Tower => 900,
        },
        Building::Tower => 150,
        Building::GoldFarm => 200,
    };
    Cost { gold }
}

#[allow(clippy::type_complexity)]
fn check_building_interaction(
    lobby: Res<ServerLobby>,
    player: Query<(&Transform, &BoxCollider, &GameSceneId, &Inventory)>,
    building: Query<(
        Entity,
        &Transform,
        &BoxCollider,
        &GameSceneId,
        &Building,
        &BuildStatus,
        &Owner,
    )>,
    mut build: EventWriter<BuildingConstruction>,
    mut upgrade: EventWriter<BuildingUpgrade>,
    mut interactions: EventReader<InteractEvent>,
) {
    for event in interactions.read() {
        let client_id = event.0;
        let player_entity = lobby.players.get(&client_id).unwrap();

        let (player_transform, player_collider, player_scene, inventory) =
            player.get(*player_entity).unwrap();

        let player_bounds = player_collider.at(player_transform);

        for (
            entity,
            building_transform,
            building_collider,
            builing_scene,
            building,
            status,
            owner,
        ) in building.iter()
        {
            if player_scene.ne(builing_scene) {
                continue;
            }

            let zone_bounds = building_collider.at(building_transform);

            if player_bounds.intersects(&zone_bounds) {
                match owner.faction {
                    Faction::Player {
                        client_id: other_client_id,
                    } => {
                        if other_client_id.ne(&client_id) {
                            continue;
                        }
                    }
                    _ => continue,
                }

                let info = CommonBuildingInfo {
                    client_id,
                    player_entity: *player_entity,
                    scene_id: *player_scene,
                    entity,
                    building_type: *building,
                };

                match status {
                    BuildStatus::Marker => {
                        if !inventory.gold.ge(&construction_cost(building).gold) {
                            continue;
                        }
                        build.send(BuildingConstruction(info));
                    }
                    BuildStatus::Built => {
                        if building.can_upgrade() {
                            if !inventory
                                .gold
                                .ge(&construction_cost(&building.upgrade_building().unwrap()).gold)
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
    }
}

fn construct_building(
    mut commands: Commands,
    mut builds: EventReader<BuildingConstruction>,
    mut building: Query<(
        &mut BuildStatus,
        &Building,
        &GameSceneId,
        &SceneBuildingIndicator,
    )>,
    mut inventory: Query<&mut Inventory>,
    mut sender: EventWriter<SendServerMessage>,
) {
    for build in builds.read() {
        let (mut status, building, game_scene_id, building_indicator) =
            building.get_mut(build.0.entity).unwrap();
        *status = BuildStatus::Built;

        commands
            .entity(build.0.entity)
            .insert(building_health(&build.0.building_type));

        println!("Building constructed: {:?}", building_indicator);

        sender.send(SendServerMessage {
            message: ServerMessages::BuildingUpdate(BuildingUpdate {
                indicator: *building_indicator,
                update: UpdateType::Status {
                    new_status: *status,
                },
            }),
            game_scene_id: *game_scene_id,
        });

        let mut inventory = inventory.get_mut(build.0.player_entity).unwrap();
        inventory.gold -= construction_cost(building).gold;
    }
}

fn upgrade_building(
    mut commands: Commands,
    mut upgrade: EventReader<BuildingUpgrade>,
    mut building: Query<(&mut Building, &GameSceneId, &SceneBuildingIndicator)>,
    mut inventory: Query<&mut Inventory>,
    mut sender: EventWriter<SendServerMessage>,
) {
    for upgrade in upgrade.read() {
        let (mut building, game_scene_id, building_indicator) =
            building.get_mut(upgrade.0.entity).unwrap();

        let upgraded_building = &upgrade
            .0
            .building_type
            .upgrade_building()
            .expect("No Upgrade specified.");

        println!("Upgraded building: {:?}", upgraded_building);

        *building = *upgraded_building;

        commands
            .entity(upgrade.0.entity)
            .insert(building_health(upgraded_building))
            .insert(building_collider(upgraded_building));

        println!("Building upgraded: {:?}", building_indicator);

        sender.send(SendServerMessage {
            message: ServerMessages::BuildingUpdate(BuildingUpdate {
                indicator: *building_indicator,
                update: UpdateType::Upgrade {
                    upgraded_building: *upgraded_building,
                },
            }),
            game_scene_id: *game_scene_id,
        });

        let mut inventory = inventory.get_mut(upgrade.0.player_entity).unwrap();
        inventory.gold -= construction_cost(upgraded_building).gold;
    }
}
