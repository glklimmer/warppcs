use bevy::prelude::*;

use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy_renet::renet::{ClientId, RenetServer};

use crate::map::base::Building;
use crate::map::Layers;
use crate::networking::{
    Owner, ServerChannel, ServerMessages, SpawnFlag, SpawnUnit, Unit, UnitType,
};
use crate::{map::GameSceneId, BoxCollider};

use super::ai::attack::{unit_health, unit_swing_timer};
use super::ai::UnitBehaviour;
use super::networking::{InteractEvent, ServerLobby};
use super::physics::attachment::AttachedTo;
use super::physics::movement::Velocity;

#[derive(Event)]
struct RecruitEvent {
    client_id: ClientId,
    scene_id: GameSceneId,
    building_type: Building,
}

pub struct BuildingsPlugins;

impl Plugin for BuildingsPlugins {
    fn build(&self, app: &mut App) {
        app.add_event::<RecruitEvent>();

        app.add_systems(
            FixedUpdate,
            (check_recruit).run_if(on_event::<InteractEvent>()),
        );

        app.add_systems(FixedUpdate, recruit.run_if(on_event::<RecruitEvent>()));
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
        for (building_transform, building_collider, builing_scene, building) in building.iter() {
            if player_scene.ne(builing_scene) {
                continue;
            }
            let player_bounds = Aabb2d::new(
                player_transform.translation.truncate(),
                player_collider.half_size(),
            );
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
        };

        let unit = Unit {
            health: unit_health(&unit_type),
            swing_timer: unit_swing_timer(&unit_type),
            unit_type: unit_type.clone(),
        };
        let owner = Owner(event.client_id);

        for unit_number in 1..=4 {
            let offset = Vec2::new(40. * (unit_number - 3) as f32 + 20., 0.);
            let unit_entity = commands
                .spawn((
                    Transform::from_translation(flag_translation),
                    unit.clone(),
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
