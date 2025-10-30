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

fn movement_input(input: Res<ButtonInput<KeyCode>>, mut commands: Commands) -> Result {
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
    Ok(())
}

fn apply_movement(
    trigger: Trigger<FromClient<MovePlayer>>,
    mut players: Query<(&mut Velocity, &Speed)>,
    client_player_map: Res<ClientPlayerMap>,
) -> Result {
    let Some(player) = client_player_map.get(&trigger.client_entity) else {
        return Err(BevyError::from("Player not found"));
    };

    let (mut velocity, speed) = players.get_mut(*player)?;

    let direction = Vec2::new(trigger.event.x, 0.).normalize_or_zero();
    velocity.0 = direction * speed.0;
    Ok(())
}
