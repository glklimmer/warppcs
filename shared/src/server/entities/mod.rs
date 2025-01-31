use bevy::{math::bounding::IntersectsVolume, prelude::*};

use health::HealthPlugin;

use crate::{
    map::{Chest, GameSceneId},
    networking::{MountType, UnitType},
    unit_collider, BoxCollider,
};

use super::{
    networking::ServerLobby,
    physics::{movement::Velocity, PushBack},
    players::InteractEvent,
};

pub mod health;

#[derive(Component, Clone)]
#[require(BoxCollider(unit_collider), Velocity, PushBack)]
pub struct Unit {
    pub unit_type: UnitType,
    pub swing_timer: Timer,
}

#[derive(Component, Clone)]
#[require(BoxCollider(unit_collider), Velocity)]
pub struct Mount {
    pub mount_type: MountType,
}

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HealthPlugin);

        app.add_event::<ChestOpenEvent>();
        app.add_event::<MountEvent>();

        app.add_systems(FixedUpdate, (check_open_chest, check_mount));
    }
}

#[derive(Event)]
pub struct ChestOpenEvent {
    pub player: Entity,
}

fn check_open_chest(
    lobby: Res<ServerLobby>,
    player: Query<(&Transform, &BoxCollider, &GameSceneId)>,
    chests: Query<(&Transform, &BoxCollider, &GameSceneId), With<Chest>>,
    mut chest_open: EventWriter<ChestOpenEvent>,
    mut interactions: EventReader<InteractEvent>,
) {
    for event in interactions.read() {
        let client_id = event.0;
        let player_entity = lobby.players.get(&client_id).unwrap();

        let (player_transform, player_collider, player_scene) = player.get(*player_entity).unwrap();
        let player_bounds = player_collider.at(player_transform);

        for (chest_transform, chest_collider, chest_scene) in chests.iter() {
            if player_scene.ne(chest_scene) {
                continue;
            }
            let chest_bounds = chest_collider.at(chest_transform);
            if player_bounds.intersects(&chest_bounds) {
                chest_open.send(ChestOpenEvent {
                    player: *player_entity,
                });
            }
        }
    }
}

#[derive(Event)]
pub struct MountEvent {
    pub player: Entity,
    pub mount_type: MountType,
}

fn check_mount(
    mut interactions: EventReader<InteractEvent>,
    mut mount_interact: EventWriter<MountEvent>,
    lobby: Res<ServerLobby>,
    player: Query<(&Transform, &BoxCollider, &GameSceneId)>,
    mounts: Query<(&Transform, &BoxCollider, &GameSceneId, &Mount)>,
) {
    for event in interactions.read() {
        let client_id = event.0;
        let player_entity = lobby.players.get(&client_id).unwrap();

        let (player_transform, player_collider, player_scene) = player.get(*player_entity).unwrap();
        let player_bounds = player_collider.at(player_transform);

        for (mount_transform, mount_collider, mount_scene, mount) in mounts.iter() {
            if player_scene.ne(mount_scene) {
                continue;
            }
            let chest_bounds = mount_collider.at(mount_transform);
            if player_bounds.intersects(&chest_bounds) {
                mount_interact.send(MountEvent {
                    player: *player_entity,
                    mount_type: mount.mount_type,
                });
            }
        }
    }
}
