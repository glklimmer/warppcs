use army::{
    ArmyFlagAssignments,
    flag::{DropFlagEvent, FlagUnits, PickFlagEvent},
};
use bevy::prelude::*;

use crate::UnitBehaviour;

pub(crate) struct FlagPlugin;

impl Plugin for FlagPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                drop_flag.run_if(on_message::<DropFlagEvent>),
                pick_flag.run_if(on_message::<PickFlagEvent>),
            ),
        );
    }
}

fn pick_flag(
    mut pick_flag: MessageReader<PickFlagEvent>,
    units: Query<&FlagUnits>,
    army: Query<Option<&ArmyFlagAssignments>>,
    mut commands: Commands,
) -> Result {
    for event in pick_flag.read() {
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

fn drop_flag(
    mut drop_flag: MessageReader<DropFlagEvent>,
    units: Query<&FlagUnits>,
    army: Query<Option<&ArmyFlagAssignments>>,
    mut commands: Commands,
) -> Result {
    for event in drop_flag.read() {
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
