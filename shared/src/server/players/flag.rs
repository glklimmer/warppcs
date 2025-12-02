use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::server::{
    ai::UnitBehaviour, buildings::recruiting::FlagUnits, entities::commander::ArmyFlagAssignments,
};

use super::{
    super::{buildings::recruiting::FlagHolder, physics::attachment::AttachedTo},
    interaction::{InteractionTriggeredEvent, InteractionType},
};

#[derive(Message)]
pub struct DropFlagEvent {
    player: Entity,
    flag: Entity,
}

#[derive(Message)]
pub struct PickFlagEvent {
    player: Entity,
    flag: Entity,
}

#[derive(Component, Serialize, Deserialize)]
pub struct FlagDestroyed;

pub fn flag_interact(
    mut interactions: MessageReader<InteractionTriggeredEvent>,
    mut drop_flag: MessageWriter<DropFlagEvent>,
    mut pick_flag: MessageWriter<PickFlagEvent>,
    flag_holder: Query<Option<&FlagHolder>>,
) -> Result {
    for event in interactions.read() {
        let InteractionType::Flag = &event.interaction else {
            continue;
        };

        let player = event.player;
        let has_flag = flag_holder.get(player)?;

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
    Ok(())
}

pub fn pick_flag(
    mut pick_flag: MessageReader<PickFlagEvent>,
    mut flag_query: Query<&mut Transform>,
    units: Query<&FlagUnits>,
    army: Query<Option<&ArmyFlagAssignments>>,
    mut commands: Commands,
) -> Result {
    for event in pick_flag.read() {
        let mut transform = flag_query.get_mut(event.flag)?;

        transform.translation.y = 10.;

        commands.entity(event.flag).insert(AttachedTo(event.player));
        commands.entity(event.player).insert(FlagHolder(event.flag));

        let flag_units = units.get(event.flag)?;

        let mut all_units: Vec<Entity> = flag_units.iter().collect();
        if let Some(commander) = flag_units.iter().next()
            && let Some(army) = army.get(commander)?
        {
            for formation_flag in army.flags.iter().flatten() {
                let formation_units = units.get(*formation_flag)?;
                let units: Vec<Entity> = formation_units.iter().collect();
                all_units.append(&mut units.clone());
            }
        };

        for unit in all_units.iter() {
            commands.entity(*unit).insert(UnitBehaviour::FollowFlag);
        }
    }
    Ok(())
}

pub fn drop_flag(
    mut drop_flag: MessageReader<DropFlagEvent>,
    mut flag_query: Query<&mut Transform>,
    units: Query<&FlagUnits>,
    army: Query<Option<&ArmyFlagAssignments>>,
    mut commands: Commands,
) -> Result {
    for event in drop_flag.read() {
        let mut transform = flag_query.get_mut(event.flag)?;

        transform.translation.y = 0.;

        commands.entity(event.flag).remove::<AttachedTo>();
        commands.entity(event.player).remove::<FlagHolder>();

        let flag_units = units.get(event.flag)?;

        let mut all_units: Vec<Entity> = flag_units.iter().collect();
        let first_unit = flag_units.iter().next();
        if let Some(commander) = first_unit
            && let Some(army) = army.get(commander)?
        {
            for formation_flag in army.flags.iter().flatten() {
                let formation_units = units.get(*formation_flag)?;
                let units: Vec<Entity> = formation_units.iter().collect();
                all_units.append(&mut units.clone());
            }
        };

        for unit in all_units.iter() {
            commands.entity(*unit).insert(UnitBehaviour::Idle);
        }
    }
    Ok(())
}
