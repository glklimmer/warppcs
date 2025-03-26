use bevy::prelude::*;

use crate::server::buildings::recruiting::Flag;

use super::{
    super::{buildings::recruiting::FlagHolder, physics::attachment::AttachedTo},
    interaction::{InteractionTriggeredEvent, InteractionType},
};

#[derive(Event)]
pub struct DropFlagEvent {
    player: Entity,
    flag: Entity,
}

#[derive(Event)]
pub struct PickFlagEvent {
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
                    player: event.player,
                    flag: event.interactable,
                });
            }
            None => {
                pick_flag.send(PickFlagEvent {
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
    mut flag_query: Query<(&mut Transform), With<Flag>>,
) {
    for event in pick_flag.read() {
        match flag_query.get_mut(event.flag) {
            Ok(mut transform) => {
                transform.translation.y = 10.;
                commands.entity(event.flag).insert(AttachedTo(event.player));
            }
            Err(_) => todo!(),
        }

        commands.entity(event.player).insert(FlagHolder(event.flag));
    }
}

pub fn drop_flag(
    mut drop_flag: EventReader<DropFlagEvent>,
    mut commands: Commands,
    mut flag_query: Query<(Entity, &mut Transform, &AttachedTo)>,
) {
    for event in drop_flag.read() {
        commands.entity(event.player).remove::<FlagHolder>();

        let (flag_entity, mut transform, attachted_to) = flag_query.get_mut(event.flag).unwrap();

        if attachted_to.0.ne(&event.player) {
            continue;
        }
        commands.entity(flag_entity).remove::<AttachedTo>();
        transform.translation.y = 0.;
    }
}
