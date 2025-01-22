use bevy::{math::bounding::IntersectsVolume, prelude::*};

use bevy_renet::renet::{ClientId, RenetServer};

use crate::{
    map::GameSceneId,
    networking::{
        DropFlag, Faction, Owner, PickFlag, PlayerCommand, ServerChannel, ServerMessages,
    },
    BoxCollider,
};

use super::{
    buildings::recruiting::{Flag, FlagHolder},
    networking::{NetworkEvent, SendServerMessage, ServerLobby},
    physics::attachment::AttachedTo,
};

#[derive(Event)]
pub struct InteractEvent(pub ClientId);

#[derive(Event)]
pub struct DropFlagEvent(pub ClientId);

#[derive(Event)]
pub struct PickFlagEvent {
    client: ClientId,
    flag: Entity,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractEvent>();
        app.add_event::<DropFlagEvent>();
        app.add_event::<PickFlagEvent>();

        app.add_systems(
            FixedUpdate,
            (attack, interact, flag_interact).run_if(on_event::<NetworkEvent>),
        );
        app.add_systems(FixedUpdate, drop_flag.run_if(on_event::<DropFlagEvent>));
        app.add_systems(FixedUpdate, pick_flag.run_if(on_event::<PickFlagEvent>));
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

fn flag_interact(
    mut network_events: EventReader<NetworkEvent>,
    player: Query<(&Transform, &BoxCollider, &GameSceneId, Option<&FlagHolder>)>,
    flag: Query<(Entity, &BoxCollider, &Transform, &GameSceneId, &Owner), With<Flag>>,
    lobby: Res<ServerLobby>,
    mut drop_flag: EventWriter<DropFlagEvent>,
    mut pick_flag: EventWriter<PickFlagEvent>,
) {
    for event in network_events.read() {
        if let PlayerCommand::FlagInteract = &event.message {
            let player_entity = lobby.players.get(&event.client_id).unwrap();

            let (player_transform, player_collider, player_scene, has_flag) =
                player.get(*player_entity).unwrap();
            let player_bounds = player_collider.at(player_transform);

            for (flag_entity, flag_collider, flag_transform, flag_scene, flag_owner) in flag.iter()
            {
                if player_scene.ne(flag_scene) {
                    continue;
                }

                match flag_owner.faction {
                    Faction::Player { client_id: owner } => {
                        if owner.ne(&event.client_id) {
                            continue;
                        }
                    }
                    Faction::Bandits => (),
                }

                let flag_bounds = flag_collider.at(flag_transform);

                if player_bounds.intersects(&flag_bounds) {
                    match has_flag {
                        Some(_) => {
                            drop_flag.send(DropFlagEvent(event.client_id));
                        }
                        None => {
                            pick_flag.send(PickFlagEvent {
                                client: event.client_id,
                                flag: flag_entity,
                            });
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn drop_flag(
    mut drop_flag: EventReader<DropFlagEvent>,
    mut commands: Commands,
    mut flag_query: Query<(Entity, &mut Transform, &AttachedTo)>,
    mut server: ResMut<RenetServer>,
    lobby: Res<ServerLobby>,
) {
    for event in drop_flag.read() {
        let client_id = event.0;
        let player_entity = lobby.players.get(&client_id).unwrap();

        commands.entity(*player_entity).remove::<FlagHolder>();

        for (flag_entity, mut transform, attachted_to) in flag_query.iter_mut() {
            if attachted_to.0.ne(player_entity) {
                continue;
            }
            commands.entity(flag_entity).remove::<AttachedTo>();
            transform.translation.y = 0.;

            let message = ServerMessages::DropFlag(DropFlag {
                entity: flag_entity,
                translation: transform.translation,
            });
            let message = bincode::serialize(&message).unwrap();
            server.send_message(client_id, ServerChannel::ServerMessages, message);
        }
    }
}

fn pick_flag(
    mut commands: Commands,
    mut pick_flag: EventReader<PickFlagEvent>,
    lobby: Res<ServerLobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in pick_flag.read() {
        let player_entity = lobby.players.get(&event.client).unwrap();
        commands
            .entity(event.flag)
            .insert(AttachedTo(*player_entity));

        commands
            .entity(*player_entity)
            .insert(FlagHolder(event.flag));

        let message = ServerMessages::PickFlag(PickFlag { entity: event.flag });
        let message = bincode::serialize(&message).unwrap();
        server.send_message(event.client, ServerChannel::ServerMessages, message);
    }
}
