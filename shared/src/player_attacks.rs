use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{
    AnimationChange, AnimationChangeEvent, ClientPlayerMap, ClientPlayerMapExt,
    networking::WorldDirection,
    server::{
        ai::UnitBehaviour,
        buildings::recruiting::{FlagHolder, FlagUnits},
        entities::commander::ArmyFlagAssignments,
    },
};

#[derive(Component)]
pub struct AttackIndicator {
    pub direction: WorldDirection,
}

pub struct PlayerAttacks;

impl Plugin for PlayerAttacks {
    fn build(&self, app: &mut App) {
        app.add_client_message::<Attack>(Channel::Ordered)
            .add_observer(attack)
            .add_systems(Update, attack_input.before(ClientSet::Send));
    }
}

#[derive(Deserialize, Serialize, Event)]
struct Attack;

fn attack_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut commands: Commands) -> Result {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        commands.client_trigger(Attack);
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
    let player = client_player_map.get_player(&trigger.client_entity)?;
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
            commands.entity(flag).insert(AttackIndicator { direction });
        }
        UnitBehaviour::FollowFlag | UnitBehaviour::Idle => {
            commands.entity(flag).remove::<AttackIndicator>();
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
