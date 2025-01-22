use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use bevy_renet::renet::{ClientId, RenetServer};

use crate::{
    flag_collider,
    map::{
        buildings::{BuildStatus, Building, RecruitmentBuilding},
        GameSceneId, Layers,
    },
    networking::{
        Faction, Inventory, Owner, ServerChannel, ServerMessages, SpawnFlag, SpawnUnit, UnitType,
    },
    server::{
        ai::{
            attack::{unit_health, unit_swing_timer},
            UnitBehaviour,
        },
        entities::{health::Health, Unit},
        networking::ServerLobby,
        physics::attachment::AttachedTo,
        players::InteractEvent,
    },
    BoxCollider,
};

#[derive(Component)]
#[require(BoxCollider(flag_collider))]
pub struct Flag;

/// PlayerEntity is FlagHolder
#[derive(Component)]
pub struct FlagHolder(pub Entity);

#[derive(Component)]
pub struct FlagAssignment(pub Entity, pub Vec2);

#[derive(Event)]
pub struct RecruitEvent {
    client_id: ClientId,
    scene_id: GameSceneId,
    building_type: Building,
}

pub fn recruit(
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
        let owner = Owner {
            faction: Faction::Player {
                client_id: event.client_id,
            },
        };

        let flag_entity = commands
            .spawn((
                Flag,
                Transform::from_translation(Vec3::ZERO),
                AttachedTo(*player_entity),
                owner,
                event.scene_id,
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
            Building::Wall { level: _ }
            | Building::Tower
            | Building::GoldFarm
            | Building::MainBuilding { level: _ } => continue,
        };

        let unit = Unit {
            swing_timer: unit_swing_timer(&unit_type),
            unit_type,
        };
        let health = Health {
            hitpoints: unit_health(&unit_type),
        };

        for unit_number in 1..=4 {
            let offset = Vec2::new(40. * (unit_number - 3) as f32 + 20., 0.);
            let unit_entity = commands
                .spawn((
                    Transform::from_translation(flag_translation),
                    unit.clone(),
                    health.clone(),
                    owner,
                    FlagAssignment(flag_entity, offset),
                    UnitBehaviour::FollowFlag(flag_entity, offset),
                    event.scene_id,
                ))
                .id();
            let message = ServerMessages::SpawnUnit(SpawnUnit {
                entity: unit_entity,
                owner,
                unit_type,
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

#[allow(clippy::type_complexity)]
pub fn check_recruit(
    lobby: Res<ServerLobby>,
    player: Query<(&Transform, &BoxCollider, &GameSceneId, &Inventory)>,
    building: Query<
        (
            &Transform,
            &BoxCollider,
            &GameSceneId,
            &Building,
            &BuildStatus,
            &Owner,
        ),
        With<RecruitmentBuilding>,
    >,
    mut recruit: EventWriter<RecruitEvent>,
    mut interactions: EventReader<InteractEvent>,
) {
    for event in interactions.read() {
        let client_id = event.0;
        let player_entity = lobby.players.get(&client_id).unwrap();

        let (player_transform, player_collider, player_scene, inventory) =
            player.get(*player_entity).unwrap();

        let player_bounds = player_collider.at(player_transform);

        for (
            building_transform,
            building_collider,
            builing_scene,
            building,
            building_status,
            building_owner,
        ) in building.iter()
        {
            match building_owner.faction {
                Faction::Player {
                    client_id: other_client_id,
                } => {
                    if other_client_id.ne(&client_id) {
                        continue;
                    }
                }
                _ => continue,
            }
            if player_scene.ne(builing_scene) {
                continue;
            }
            if BuildStatus::Built.ne(building_status) {
                continue;
            }

            let gold_cost: u16 = match building {
                Building::MainBuilding { level: _ }
                | Building::Wall { level: _ }
                | Building::Tower
                | Building::GoldFarm => continue,
                Building::Archer => 10,
                Building::Warrior => 10,
                Building::Pikeman => 10,
            };
            if !inventory.gold.gt(&gold_cost) {
                continue;
            }

            let zone_bounds = building_collider.at(building_transform);
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
