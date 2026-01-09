use bevy::prelude::*;
use bevy_replicon::prelude::*;

use physics::attachment::AttachmentTilting;
use serde::{Deserialize, Serialize};

use ai::UnitBehaviour;
use army::{
    ArmyFlagAssignments,
    flag::{FlagHolder, FlagUnits},
};
use lobby::{ClientPlayerMap, ClientPlayerMapExt};
use shared::{AnimationChange, AnimationChangeEvent};

pub(crate) struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_event::<Attack>(Channel::Ordered)
            .add_observer(attack)
            .add_systems(Update, attack_input.before(ClientSystems::Send));
    }
}

#[derive(Deserialize, Serialize, Event)]
struct Attack(usize);

fn attack_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut commands: Commands) -> Result {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        commands.client_trigger(Attack(0));
    }
    Ok(())
}

fn attack(
    trigger: On<FromClient<Attack>>,
    mut animation: MessageWriter<ToClients<AnimationChangeEvent>>,
    flag_holder: Query<(Option<&FlagHolder>, &Transform)>,
    units: Query<&FlagUnits>,
    army: Query<&ArmyFlagAssignments>,
    behaviour: Query<&UnitBehaviour>,
    client_player_map: Res<ClientPlayerMap>,
    mut commands: Commands,
) -> Result {
    let player = client_player_map.get_player(&trigger.client_id)?;
    let (maybe_flag_holder, transform) = flag_holder.get(*player)?;

    let Some(flag_holder) = maybe_flag_holder else {
        animation.write(ToClients {
            mode: SendMode::Broadcast,
            message: AnimationChangeEvent {
                entity: *player,
                change: AnimationChange::Attack,
            },
        });
        return Ok(());
    };

    let flag = **flag_holder;
    let flag_units = units.get(flag)?;

    let first_unit = flag_units.iter().next();
    let Some(unit) = first_unit else {
        return Ok(());
    };

    let behaviour = behaviour.get(unit)?;
    let new_behaviour = match behaviour {
        UnitBehaviour::Attack(_) => UnitBehaviour::FollowFlag,
        UnitBehaviour::FollowFlag | UnitBehaviour::Idle => {
            UnitBehaviour::Attack(transform.scale.x.into())
        }
    };

    match new_behaviour {
        UnitBehaviour::Attack(direction) => {
            commands
                .entity(flag)
                .insert(AttachmentTilting { direction });
        }
        UnitBehaviour::FollowFlag | UnitBehaviour::Idle => {
            commands.entity(flag).remove::<AttachmentTilting>();
        }
    }

    let mut all_units: Vec<Entity> = flag_units.iter().collect();
    if let Ok(army) = army.get(unit) {
        for formation_flag in army.flags.iter().flatten() {
            let formation_units = units.get(*formation_flag)?;
            let units: Vec<Entity> = formation_units.iter().collect();
            all_units.append(&mut units.clone());
        }
    }

    for unit in all_units.iter() {
        commands.entity(*unit).insert(new_behaviour.clone());
    }
    Ok(())
}
