use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{
    AnimationChange, AnimationChangeEvent, ClientPlayerMap,
    server::{
        ai::UnitBehaviour,
        buildings::recruiting::{FlagHolder, FlagUnits},
    },
};

pub struct PlayerAttacks;

impl Plugin for PlayerAttacks {
    fn build(&self, app: &mut App) {
        app.add_client_trigger::<Attack>(Channel::Ordered)
            .add_observer(attack)
            .add_systems(Update, attack_input.before(ClientSet::Send));
    }
}

#[derive(Deserialize, Serialize, Event)]
struct Attack;

fn attack_input(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        commands.client_trigger(Attack);
    }
}

fn attack(
    trigger: Trigger<FromClient<Attack>>,
    mut animation: EventWriter<ToClients<AnimationChangeEvent>>,
    mut commands: Commands,
    flag_holder: Query<(Option<&FlagHolder>, &Transform)>,
    units: Query<&FlagUnits>,
    behaviour: Query<&UnitBehaviour>,
    client_player_map: Res<ClientPlayerMap>,
) {
    let player = client_player_map.get(&trigger.client_entity).unwrap();

    let (maybe_flag_holder, transform) = flag_holder.get(*player).unwrap();

    let Some(flag_holder) = maybe_flag_holder else {
        animation.write(ToClients {
            mode: SendMode::Broadcast,
            event: AnimationChangeEvent {
                entity: *player,
                change: AnimationChange::Attack,
            },
        });
        return;
    };

    let flag = **flag_holder;
    let units = units.get(flag).unwrap();

    let first_unit = units.iter().next();
    let Some(unit) = first_unit else {
        return;
    };
    let behaviour = behaviour.get(unit).unwrap();
    let new_behaviour = match behaviour {
        UnitBehaviour::Idle => UnitBehaviour::Attack(transform.scale.x.into()),
        UnitBehaviour::Attack(_) => UnitBehaviour::Idle,
    };

    for unit in units.iter() {
        commands.entity(unit).insert(new_behaviour.clone());
    }
}
