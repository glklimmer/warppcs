use bevy::prelude::*;

use bevy_replicon::prelude::ServerState;
use health::{DelayedDespawn, Health};
use interaction::{Interactable, InteractionType};
use physics::{attachment::AttachedTo, movement::Velocity};
use shared::Owner;
use units::Unit;

use crate::{
    ArmyFlagAssignments,
    flag::{FlagAssignment, FlagDestroyed, FlagHolder, FlagUnits},
};

pub(crate) struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_unit_death);
    }
}

fn on_unit_death(
    death: On<Remove, Health>,
    units: Query<
        (
            &Owner,
            Option<&FlagAssignment>,
            Option<&ArmyFlagAssignments>,
        ),
        With<Unit>,
    >,
    group: Query<&FlagUnits>,
    transform: Query<&Transform>,
    holder: Query<&FlagHolder>,
    server_state: Res<State<ServerState>>,
    mut commands: Commands,
) -> Result {
    let ServerState::Running = server_state.get() else {
        return Ok(());
    };
    let entity = death.entity;
    let Ok((owner, maybe_flag_assignment, maybe_army)) = units.get(entity) else {
        return Ok(());
    };

    commands.entity(entity).try_remove::<Interactable>();

    let Some(flag_assignment) = maybe_flag_assignment else {
        return Ok(());
    };

    commands.entity(entity).remove::<FlagAssignment>();

    let flag = flag_assignment.entity();
    let group = group.get(flag)?;
    let num_alive = group.len();

    // last unit from flag died
    if num_alive == 1 {
        let flag_transform = transform.get(flag)?;

        commands
            .entity(flag)
            .insert((
                DelayedDespawn(Timer::from_seconds(620., TimerMode::Once)),
                FlagDestroyed,
            ))
            .remove::<AttachedTo>()
            .remove::<Interactable>();

        let Ok(player) = owner.entity() else {
            return Ok(());
        };

        if let Ok(holder) = holder.get(player)
            && flag.eq(&**holder)
        {
            commands.entity(player).remove::<FlagHolder>();
        }

        if let Some(army) = maybe_army {
            for formation_flag in army.flags.iter().flatten() {
                commands.entity(*formation_flag).remove::<AttachedTo>();
                commands.entity(*formation_flag).insert((
                    *flag_transform,
                    Velocity(Vec2::new((fastrand::f32() - 0.5) * 150., 100.)),
                    Visibility::Visible,
                    Interactable {
                        kind: InteractionType::Flag,
                        restricted_to: Some(player),
                    },
                ));
            }
        }
    }

    Ok(())
}
