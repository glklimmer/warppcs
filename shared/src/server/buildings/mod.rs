use bevy::prelude::*;

use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy_renet::renet::{ClientId, RenetServer};
use gold_farm::{enable_goldfarm, gold_farm_output};
use recruiting::{check_recruit, recruit, RecruitEvent};

use crate::map::buildings::{BuildStatus, Building, Cost};
use crate::map::scenes::base::SceneBuildingIndicator;
use crate::networking::{
    BuildingUpdate, Inventory, MultiplayerRoles, Owner, ServerChannel, ServerMessages,
};
use crate::GameState;
use crate::{map::GameSceneId, BoxCollider};

use super::networking::ServerLobby;
use super::players::InteractEvent;

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
                (check_recruit, check_building_interaction).run_if(on_event::<InteractEvent>()),
                (
                    (construct_building, enable_goldfarm)
                        .run_if(on_event::<BuildingConstruction>()),
                    recruit.run_if(on_event::<RecruitEvent>()),
                ),
            )
                .chain(),
        );

        app.add_systems(
            FixedUpdate,
            gold_farm_output.run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
    }
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

        let player_bounds = Aabb2d::new(
            player_transform.translation.truncate(),
            player_collider.half_size(),
        );

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

            let zone_bounds = Aabb2d::new(
                building_transform.translation.truncate(),
                building_collider.half_size(),
            );

            if player_bounds.intersects(&zone_bounds) {
                if owner.0.ne(&client_id) {
                    continue;
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
                }
            }
        }
    }
}

fn construct_building(
    mut builds: EventReader<BuildingConstruction>,
    mut building: Query<(
        &mut BuildStatus,
        &Cost,
        &GameSceneId,
        &SceneBuildingIndicator,
    )>,
    mut inventory: Query<&mut Inventory>,
    mut server: ResMut<RenetServer>,
    lobby: Res<ServerLobby>,
    scene_ids: Query<&GameSceneId>,
) {
    for build in builds.read() {
        let (mut status, cost, game_scene_id, building_indicator) =
            building.get_mut(build.0.entity).unwrap();
        *status = BuildStatus::Built;

        println!("Building constructed: {:?}", building_indicator);

        let message = ServerMessages::BuildingUpdate(BuildingUpdate {
            indicator: *building_indicator,
            status: *status,
        });
        let message = bincode::serialize(&message).unwrap();
        for (other_client_id, other_entity) in lobby.players.iter() {
            let other_scene_id = scene_ids.get(*other_entity).unwrap();
            if game_scene_id.eq(other_scene_id) {
                server.send_message(
                    *other_client_id,
                    ServerChannel::ServerMessages,
                    message.clone(),
                );
            }
        }

        let mut inventory = inventory.get_mut(build.0.player_entity).unwrap();
        inventory.gold -= cost.gold;

        let message = ServerMessages::SyncInventory(inventory.clone());
        let message = bincode::serialize(&message).unwrap();
        server.send_message(build.0.client_id, ServerChannel::ServerMessages, message);
    }
}
