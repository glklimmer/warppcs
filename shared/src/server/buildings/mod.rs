use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use bevy_renet::renet::{ClientId, RenetServer};
use gold_farm::{enable_goldfarm, gold_farm_output};
use recruiting::{check_recruit, recruit, RecruitEvent};

use crate::networking::Faction;
use crate::{
    map::{
        buildings::{BuildStatus, Building, Cost},
        scenes::SceneBuildingIndicator,
        GameSceneId,
    },
    networking::{
        BuildingUpdate, Inventory, MultiplayerRoles, Owner, ServerChannel, ServerMessages,
    },
    BoxCollider, GameState,
};

use super::networking::SendServerMessage;
use super::{entities::health::Health, networking::ServerLobby, players::InteractEvent};

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
                    recruit.run_if(on_event::<RecruitEvent>),
                ),
            )
                .chain(),
        );

        app.add_systems(
            FixedUpdate,
            gold_farm_output
                .run_if(in_state(GameState::GameSession).and(in_state(MultiplayerRoles::Host))),
        );
    }
}

pub fn building_health(building_type: &Building) -> Health {
    let hitpoints = match building_type {
        Building::MainBuilding => 1200.,
        Building::Archer => 800.,
        Building::Warrior => 800.,
        Building::Pikeman => 800.,
        Building::Wall => 600.,
        Building::Tower => 400.,
        Building::GoldFarm => 600.,
    };
    Health { hitpoints }
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
        &Cost,
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
            cost,
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

                if !inventory.gold.gt(&cost.gold) {
                    continue;
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
                        build.send(BuildingConstruction(info));
                    }
                    BuildStatus::Built => {
                        upgrade.send(BuildingUpgrade(info));
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
        &Cost,
        &GameSceneId,
        &SceneBuildingIndicator,
    )>,
    mut inventory: Query<&mut Inventory>,
    mut server: ResMut<RenetServer>,
    mut sender: EventWriter<SendServerMessage>,
) {
    for build in builds.read() {
        let (mut status, cost, game_scene_id, building_indicator) =
            building.get_mut(build.0.entity).unwrap();
        *status = BuildStatus::Built;

        commands
            .entity(build.0.entity)
            .insert(building_health(&build.0.building_type));

        println!("Building constructed: {:?}", building_indicator);

        sender.send(SendServerMessage {
            message: ServerMessages::BuildingUpdate(BuildingUpdate {
                indicator: *building_indicator,
                status: *status,
            }),
            game_scene_id: *game_scene_id,
        });

        let mut inventory = inventory.get_mut(build.0.player_entity).unwrap();
        inventory.gold -= cost.gold;

        let message = ServerMessages::SyncInventory(inventory.clone());
        let message = bincode::serialize(&message).unwrap();
        server.send_message(build.0.client_id, ServerChannel::ServerMessages, message);
    }
}
