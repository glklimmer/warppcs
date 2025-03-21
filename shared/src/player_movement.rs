use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{
    server::physics::movement::{Speed, Velocity},
    PhysicalPlayer,
};

pub struct PlayerMovement;

impl Plugin for PlayerMovement {
    fn build(&self, app: &mut App) {
        app.add_client_trigger::<MovePlayer>(ChannelKind::Ordered)
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
    mut players: Query<(&PhysicalPlayer, &mut Velocity, &mut Transform, &Speed)>,
) {
    for (player, mut velocity, mut transform, speed) in &mut players {
        if trigger.client_id == **player {
            let direction = Vec2::new(trigger.event.x, 0.).normalize_or_zero();
            velocity.0 = direction * speed.0;
            transform.scale.x = direction.x.signum();
        }
    }
}

// fn apply_movement(
//     trigger: Trigger<FromClient<MovePlayer>>,
//     time: Res<Time>,
//     mut boxes: Query<(&PhysicalPlayer, &mut Transform)>,
// ) {
//     const MOVE_SPEED: f32 = 300.0;
//     info!("received movement from `{:?}`", trigger.client_id);
//     for (player, mut transform) in &mut boxes {
//         if trigger.client_id == **player {
//             transform.translation +=
//                 Vec3::new(trigger.event.x, trigger.event.y, 0.0) * time.delta_secs() * MOVE_SPEED;
//         }
//     }
// }
