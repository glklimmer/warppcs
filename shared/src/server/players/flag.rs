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
    flag_holder: Query<Option<&FlagHolder>>,
) {
    for event in interactions.read() {
        let InteractionType::Flag = &event.interaction else {
            continue;
        };
        let player = event.player;

        let has_flag = flag_holder.get(player).unwrap();

        match has_flag {
            Some(_) => {
                drop_flag.send(DropFlagEvent {
                    player,
                    flag: event.interactable,
                });
            }
            None => {
                pick_flag.send(PickFlagEvent {
                    player,
                    flag: event.interactable,
                });
            }
        }
    }
}

pub fn pick_flag(
    mut commands: Commands,
    mut pick_flag: EventReader<PickFlagEvent>,
    mut flag_query: Query<&mut Transform, With<Flag>>,
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
    mut flag_query: Query<(Entity, &mut Transform, Option<&AttachedTo>)>,
) {
    for event in drop_flag.read() {
        let (flag_entity, mut transform, maybe_attachted) = flag_query.get_mut(event.flag).unwrap();

        if maybe_attachted
            .as_ref()
            .map_or(true, |attached| attached.0 != event.player)
        {
            continue;
        }
        commands.entity(event.player).remove::<FlagHolder>();
        commands.entity(flag_entity).remove::<AttachedTo>();
        transform.translation.y = 0.;
    }
}
