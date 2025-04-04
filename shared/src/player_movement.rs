use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{
    Player,
    server::physics::movement::{Speed, Velocity},
};

pub struct PlayerMovement;

impl Plugin for PlayerMovement {
    fn build(&self, app: &mut App) {
        app.add_client_trigger::<MovePlayer>(Channel::Ordered)
            .add_observer(apply_movement)
            .add_systems(Update, movement_input.before(ClientSet::Send));
    }
}

#[derive(Deserialize, Deref, Event, Serialize)]
struct MovePlayer(Vec2);

fn movement_input(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    let mut direction = Vec2::ZERO;
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        commands.client_trigger(MovePlayer(direction.normalize_or_zero()));
    }
}

fn apply_movement(
    trigger: Trigger<FromClient<MovePlayer>>,
    mut players: Query<(&Player, &mut Velocity, &mut Transform, &Speed)>,
) {
    let (_, mut velocity, mut transform, speed) = players
        .iter_mut()
        .find(|&(player, _, _, _)| **player == trigger.entity())
        .unwrap_or_else(|| panic!("`{}` should be connected", trigger.client_entity));

    let direction = Vec2::new(trigger.event.x, 0.).normalize_or_zero();
    velocity.0 = direction * speed.0;
    transform.scale.x = direction.x.signum();
}
