use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{
    ClientPlayerMap, PlayerState,
    server::physics::movement::{Speed, Velocity},
};

pub struct PlayerMovement;

impl Plugin for PlayerMovement {
    fn build(&self, app: &mut App) {
        app.add_client_trigger::<MovePlayer>(Channel::Ordered)
            .add_observer(apply_movement)
            .add_systems(
                Update,
                movement_input
                    .before(ClientSet::Send)
                    .run_if(in_state(PlayerState::World)),
            );
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
    mut players: Query<(&mut LinearVelocity, &mut Transform, &Speed)>,
    client_player_map: Res<ClientPlayerMap>,
) {
    let Ok((mut velocity, mut transform, speed)) =
        players.get_mut(*client_player_map.get(&trigger.client_entity).unwrap())
    else {
        return;
    };

    let direction = Vec2::new(trigger.event.x, 0.).normalize_or_zero();
    velocity.0 = direction * speed.0;
    transform.scale.x = direction.x.signum();
}
