use bevy::prelude::*;

use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy_renet::renet::{ClientId, RenetServer};

use crate::map::buildings::{BuildStatus, Building, Cost};
use crate::map::Layers;
use crate::networking::{Owner, ServerChannel, ServerMessages, SpawnFlag, SpawnUnit, UnitType};
use crate::{map::GameSceneId, BoxCollider};

use super::ai::attack::{unit_health, unit_swing_timer};
use super::ai::UnitBehaviour;
use super::economy::Inventory;
use super::entities::health::Health;
use super::entities::Unit;
use super::networking::ServerLobby;
use super::physics::attachment::AttachedTo;
use super::physics::movement::Velocity;
use super::players::InteractEvent;

#[derive(Event)]
struct RecruitEvent {
    client_id: ClientId,
    scene_id: GameSceneId,
    building_type: Building,
}

pub struct CommonBuildingInfo {
    pub client_id: ClientId,
    pub player_entity: Entity,
    pub scene_id: GameSceneId,
    pub entity: Entity,
    pub building_type: Building,
}

#[derive(Event)]
pub struct BuildingConstruction(pub CommonBuildingInfo);

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
            (check_recruit, check_building_interaction).run_if(on_event::<InteractEvent>()),
        );
        app.add_systems(
            FixedUpdate,
            construct_building.run_if(on_event::<BuildingConstruction>()),
        );

        app.add_systems(FixedUpdate, recruit.run_if(on_event::<RecruitEvent>()));
    }
}

#[allow(clippy::type_complexity)]
fn check_building_interaction(
    lobby: Res<ServerLobby>,
    player: Query<(&Transform, &BoxCollider, &GameSceneId)>,
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
    inventory: Query<&Inventory>,
    mut build: EventWriter<BuildingConstruction>,
    mut upgrade: EventWriter<BuildingUpgrade>,
    mut interactions: EventReader<InteractEvent>,
) {
    for event in interactions.read() {
        let client_id = event.0;
        let player_entity = lobby.players.get(&client_id).unwrap();

        let (player_transform, player_collider, player_scene) = player.get(*player_entity).unwrap();

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

                let inventory = inventory.get(*player_entity).unwrap();
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
    mut building: Query<(&mut BuildStatus, &Cost)>,
    mut inventory: Query<&mut Inventory>,
    mut server: ResMut<RenetServer>,
) {
    for build in builds.read() {
        let (mut status, cost) = building.get_mut(build.0.entity).unwrap();
        *status = BuildStatus::Built;

        // TODO: send building construction to clients

        let mut inventory = inventory.get_mut(build.0.player_entity).unwrap();
        inventory.gold -= cost.gold;

        let message = ServerMessages::SyncInventory(inventory.clone());
        let message = bincode::serialize(&message).unwrap();
        server.send_message(build.0.client_id, ServerChannel::ServerMessages, message);
    }
}

fn check_recruit(
    lobby: Res<ServerLobby>,
    player: Query<(&Transform, &BoxCollider, &GameSceneId)>,
    building: Query<(&Transform, &BoxCollider, &GameSceneId, &Building)>,
    mut recruit: EventWriter<RecruitEvent>,
    mut interactions: EventReader<InteractEvent>,
) {
    for event in interactions.read() {
        let client_id = event.0;
        let player_entity = lobby.players.get(&client_id).unwrap();

        let (player_transform, player_collider, player_scene) = player.get(*player_entity).unwrap();

        let player_bounds = Aabb2d::new(
            player_transform.translation.truncate(),
            player_collider.half_size(),
        );

        for (building_transform, building_collider, builing_scene, building) in building.iter() {
            if player_scene.ne(builing_scene) {
                continue;
            }

            let zone_bounds = Aabb2d::new(
                building_transform.translation.truncate(),
                building_collider.half_size(),
            );
            if player_bounds.intersects(&zone_bounds) {
                recruit.send(RecruitEvent {
                    client_id,
                    scene_id: *player_scene,
                    building_type: *building,
                });
            }
        }
    }
}

#[derive(Component)]
pub struct FlagHolder(pub Entity);

#[derive(Component)]
pub struct FlagAssignment(pub Entity, pub Vec2);

fn recruit(
    mut commands: Commands,
    mut recruit: EventReader<RecruitEvent>,
    mut server: ResMut<RenetServer>,
    lobby: Res<ServerLobby>,
    transforms: Query<&Transform>,
    scene_ids: Query<&GameSceneId>,
) {
    for event in recruit.read() {
        let player_entity = lobby.players.get(&event.client_id).unwrap();
        let player_transform = transforms.get(*player_entity).unwrap();
        let player_translation = player_transform.translation;
        let flag_translation = Vec3::new(
            player_translation.x,
            player_translation.y,
            Layers::Flag.as_f32(),
        );
        let flag_entity = commands
            .spawn((
                Transform::from_translation(Vec3::ZERO),
                AttachedTo(*player_entity),
            ))
            .id();
        commands
            .entity(*player_entity)
            .insert(FlagHolder(flag_entity));

        let message = ServerMessages::SpawnFlag(SpawnFlag {
            entity: flag_entity,
        });
        let message = bincode::serialize(&message).unwrap();
        server.send_message(
            event.client_id,
            ServerChannel::ServerMessages,
            message.clone(),
        );

        let unit_type = match event.building_type {
            Building::Archer => UnitType::Archer,
            Building::Warrior => UnitType::Shieldwarrior,
            Building::Pikeman => UnitType::Pikeman,
            Building::Wall | Building::Tower | Building::GoldFarm => continue,
        };

        let unit = Unit {
            swing_timer: unit_swing_timer(&unit_type),
            unit_type: unit_type.clone(),
        };
        let health = Health {
            hitpoints: unit_health(&unit_type),
        };
        let owner = Owner(event.client_id);

        for unit_number in 1..=4 {
            let offset = Vec2::new(40. * (unit_number - 3) as f32 + 20., 0.);
            let unit_entity = commands
                .spawn((
                    Transform::from_translation(flag_translation),
                    unit.clone(),
                    health.clone(),
                    owner,
                    Velocity::default(),
                    FlagAssignment(flag_entity, offset),
                    UnitBehaviour::FollowFlag(flag_entity, offset),
                    BoxCollider(Vec2::new(50., 90.)),
                    event.scene_id,
                ))
                .id();
            let message = ServerMessages::SpawnUnit(SpawnUnit {
                entity: unit_entity,
                owner,
                unit_type: unit_type.clone(),
                translation: player_transform.translation.into(),
            });
            let message = bincode::serialize(&message).unwrap();
            for (client_id, entity) in lobby.players.iter() {
                let player_scene_id = scene_ids.get(*entity).unwrap();
                if event.scene_id.eq(player_scene_id) {
                    server.send_message(*client_id, ServerChannel::ServerMessages, message.clone());
                }
            }
        }
    }
}
