use bevy::prelude::*;

use crate::server::{
    ai::UnitBehaviour, buildings::recruiting::FlagUnits, entities::commander::ArmyFlagAssignments,
};

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
                drop_flag.write(DropFlagEvent {
                    player,
                    flag: event.interactable,
                });
            }
            None => {
                pick_flag.write(PickFlagEvent {
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
    mut flag_query: Query<&mut Transform>,
    units: Query<&FlagUnits>,
    army: Query<&ArmyFlagAssignments>,
) {
    for event in pick_flag.read() {
        let mut transform = flag_query.get_mut(event.flag).unwrap();

        transform.translation.y = 10.;

        commands.entity(event.flag).insert(AttachedTo(event.player));
        commands.entity(event.player).insert(FlagHolder(event.flag));

        let Ok(flag_units) = units.get(event.flag) else {
            continue;
        };

        let mut all_units: Vec<Entity> = flag_units.iter().collect();
        let first_unit = flag_units.iter().next();
        if let Some(commander) = first_unit {
            if let Ok(army) = army.get(commander) {
                for formation_flag in army.flags.iter().flatten() {
                    let formation_units = units.get(*formation_flag).unwrap();
                    let units: Vec<Entity> = formation_units.iter().collect();
                    all_units.append(&mut units.clone());
                }
            }
        };

        for unit in all_units.iter() {
            commands.entity(*unit).insert(UnitBehaviour::FollowFlag);
        }
    }
}

pub fn drop_flag(
    mut drop_flag: EventReader<DropFlagEvent>,
    mut commands: Commands,
    mut flag_query: Query<&mut Transform>,
    units: Query<&FlagUnits>,
    army: Query<&ArmyFlagAssignments>,
) {
    for event in drop_flag.read() {
        let mut transform = flag_query.get_mut(event.flag).unwrap();

        transform.translation.y = 0.;

        commands.entity(event.flag).remove::<AttachedTo>();
        commands.entity(event.player).remove::<FlagHolder>();

        let Ok(flag_units) = units.get(event.flag) else {
            continue;
        };

        let mut all_units: Vec<Entity> = flag_units.iter().collect();
        let first_unit = flag_units.iter().next();
        if let Some(commander) = first_unit {
            if let Ok(army) = army.get(commander) {
                for formation_flag in army.flags.iter().flatten() {
                    let formation_units = units.get(*formation_flag).unwrap();
                    let units: Vec<Entity> = formation_units.iter().collect();
                    all_units.append(&mut units.clone());
                }
            }
        };

        for unit in all_units.iter() {
            commands.entity(*unit).insert(UnitBehaviour::Idle);
        }
    }
}
