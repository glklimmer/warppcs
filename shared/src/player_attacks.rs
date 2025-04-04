use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{AnimationChange, AnimationChangeEvent};

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
) {
    animation.send(ToClients {
        mode: SendMode::Broadcast,
        event: AnimationChangeEvent {
            entity: trigger.entity(),
            change: AnimationChange::Attack,
        },
    });
}
