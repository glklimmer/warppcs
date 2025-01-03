use bevy::{math::bounding::IntersectsVolume, prelude::*};

use health::HealthPlugin;

use crate::{networking::UnitType, unit_collider, BoxCollider};

use super::physics::{movement::Velocity, PushBack};

pub mod health;

#[derive(Component, Clone)]
#[require(BoxCollider(unit_collider), Velocity, PushBack)]
pub struct Unit {
    pub unit_type: UnitType,
    pub swing_timer: Timer,
}

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HealthPlugin);

        app.add_event::<ChestOpenEvent>();

        app.add_systems(
            FixedUpdate,
            (check_open_chest)
                .run_if(in_state(GameState::GameSession).and(in_state(MultiplayerRoles::Host))),
        );
    }
}

#[derive(Event)]
pub struct ChestOpenEvent {
    pub entity: Entity,
}

fn check_open_chest(
    lobby: Res<ServerLobby>,
    player: Query<(&Transform, &BoxCollider, &GameSceneId)>,
    chests: Query<(&Transform, &BoxCollider, &GameSceneId), With<Chest>>,
    mut travel: EventWriter<ChestOpenEvent>,
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
                travel.send(ChestOpenEvent {
                    entity: *player_entity,
                });
            }
        }
    }
}
