use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{AnimationChange, AnimationChangeEvent, ClientPlayerMap};

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
    client_player_map: Res<ClientPlayerMap>,
) {
    let player = client_player_map.get(&trigger.client_entity).unwrap();

    animation.send(ToClients {
        mode: SendMode::Broadcast,
        event: AnimationChangeEvent {
            entity: *player,
            change: AnimationChange::Attack,
        },
    });
}
