use bevy::prelude::*;

use bevy_renet::renet::ClientId;

use crate::{
    map::GameSceneId,
    networking::{DropFlag, PlayerCommand, ServerMessages},
};

use super::{
    buildings::recruiting::FlagHolder,
    networking::{NetworkEvent, SendServerMessage, ServerLobby},
    physics::attachment::AttachedTo,
};

#[derive(Event)]
pub struct InteractEvent(pub ClientId);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractEvent>();

        app.add_systems(
            FixedUpdate,
            (attack, interact, drop_flag).run_if(on_event::<NetworkEvent>),
        );
    }
}

fn attack(
    mut network_events: EventReader<NetworkEvent>,
    mut sender: EventWriter<SendServerMessage>,
    scene_ids: Query<&GameSceneId>,
    lobby: Res<ServerLobby>,
) {
    for event in network_events.read() {
        let client_id = event.client_id;
        if let PlayerCommand::MeleeAttack = &event.message {
            if let Some(player_entity) = lobby.players.get(&client_id) {
                let game_scene_id = scene_ids.get(*player_entity).unwrap();
                sender.send(SendServerMessage {
                    message: ServerMessages::MeleeAttack {
                        entity: *player_entity,
                    },
                    game_scene_id: *game_scene_id,
                });
            }
        }
    }
}

fn interact(
    mut network_events: EventReader<NetworkEvent>,
    mut interact: EventWriter<InteractEvent>,
) {
    for event in network_events.read() {
        let client_id = event.client_id;
        if let PlayerCommand::Interact = &event.message {
            interact.send(InteractEvent(client_id));
        }
    }
}

fn drop_flag(
    mut commands: Commands,
    mut network_events: EventReader<NetworkEvent>,
    mut flag_query: Query<(Entity, &AttachedTo, &mut Transform)>,
    mut flagholder_query: Query<(&FlagHolder)>,
    lobby: Res<ServerLobby>,
    mut sender: EventWriter<SendServerMessage>,
    scene_ids: Query<&GameSceneId>,
) {
    for event in network_events.read() {
        if let PlayerCommand::DropFlag = &event.message {
            let player_entity = lobby.players.get(&event.client_id).unwrap();
            let game_scene_id = scene_ids.get(*player_entity).unwrap();

            commands.entity(*player_entity).remove::<FlagHolder>();

            for (flag, attached_to, mut transform) in flag_query.iter_mut() {
                if attached_to.0.eq(player_entity) {
                    commands.entity(flag).remove::<AttachedTo>();

                    transform.translation.y = 0.;
                    transform.translation.x = 100.;

                    println!("Found Player drop");

                    sender.send(SendServerMessage {
                        message: ServerMessages::DropFlag(DropFlag { entity: flag }),
                        game_scene_id: *game_scene_id,
                    });
                }
            }
        }
    }
}
