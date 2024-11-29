use bevy::prelude::*;

use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy_renet::renet::{ClientId, RenetServer};

use crate::{
    map::{buildings::Building, GameSceneId, Layers},
    networking::{Owner, ServerChannel, ServerMessages, SpawnFlag, SpawnUnit, UnitType},
    server::{
        ai::{
            attack::{unit_health, unit_swing_timer},
            UnitBehaviour,
        },
        entities::{health::Health, Unit},
        networking::ServerLobby,
        physics::{attachment::AttachedTo, movement::Velocity},
        players::InteractEvent,
    },
    BoxCollider,
};

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

pub fn check_recruit(
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
