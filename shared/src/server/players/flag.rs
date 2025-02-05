use bevy::prelude::*;

use bevy_renet::renet::{ClientId, RenetServer};

use crate::networking::{DropFlag, PickFlag, ServerChannel, ServerMessages};

use super::{
    super::{buildings::recruiting::FlagHolder, physics::attachment::AttachedTo},
    interaction::{InteractionTriggeredEvent, InteractionType},
};

#[derive(Event)]
pub struct DropFlagEvent {
    client_id: ClientId,
    player: Entity,
    flag: Entity,
}

#[derive(Event)]
pub struct PickFlagEvent {
    client_id: ClientId,
    player: Entity,
    flag: Entity,
}

pub fn flag_interact(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut drop_flag: EventWriter<DropFlagEvent>,
    mut pick_flag: EventWriter<PickFlagEvent>,
    player: Query<Option<&FlagHolder>>,
) {
    for event in interactions.read() {
        let InteractionType::Flag = &event.interaction else {
            continue;
        };

        let has_flag = player.get(event.player).unwrap();

        match has_flag {
            Some(_) => {
                drop_flag.send(DropFlagEvent {
                    client_id: event.client_id,
                    player: event.player,
                    flag: event.interactable,
                });
            }
            None => {
                pick_flag.send(PickFlagEvent {
                    client_id: event.client_id,
                    player: event.player,
                    flag: event.interactable,
                });
            }
        }
    }
}

pub fn pick_flag(
    mut commands: Commands,
    mut pick_flag: EventReader<PickFlagEvent>,
    mut server: ResMut<RenetServer>,
) {
    for event in pick_flag.read() {
        commands.entity(event.flag).insert(AttachedTo(event.player));

        commands.entity(event.player).insert(FlagHolder(event.flag));

        let message = ServerMessages::PickFlag(PickFlag { flag: event.flag });
        let message = bincode::serialize(&message).unwrap();
        server.send_message(event.client_id, ServerChannel::ServerMessages, message);
    }
}

pub fn drop_flag(
    mut drop_flag: EventReader<DropFlagEvent>,
    mut commands: Commands,
    mut flag_query: Query<(Entity, &mut Transform, &AttachedTo)>,
    mut server: ResMut<RenetServer>,
) {
    for event in drop_flag.read() {
        commands.entity(event.player).remove::<FlagHolder>();

        let (flag_entity, mut transform, attachted_to) = flag_query.get_mut(event.flag).unwrap();

        if attachted_to.0.ne(&event.player) {
            continue;
        }
        commands.entity(flag_entity).remove::<AttachedTo>();
        transform.translation.y = 30.;

        let message = ServerMessages::DropFlag(DropFlag {
            flag: flag_entity,
            translation: transform.translation,
        });
        let message = bincode::serialize(&message).unwrap();
        server.send_message(event.client_id, ServerChannel::ServerMessages, message);
    }
}
