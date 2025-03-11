use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{PhysicalPlayer, WorldPosition};

pub struct PlayerMovement;

impl Plugin for PlayerMovement {
    fn build(&self, app: &mut App) {
        app.add_client_trigger::<MovePlayer>(ChannelKind::Ordered)
            .add_observer(apply_movement)
            .add_systems(Update, read_input.before(ClientSet::Send));
    }
}

#[derive(Deserialize, Deref, Event, Serialize)]
struct MovePlayer(Vec2);

fn read_input(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    let mut direction = Vec2::ZERO;
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        commands.client_trigger(MovePlayer(direction.normalize_or_zero()));
    }
}

fn apply_movement(
    trigger: Trigger<FromClient<MovePlayer>>,
    time: Res<Time>,
    mut boxes: Query<(&PhysicalPlayer, &mut WorldPosition)>,
) {
    const MOVE_SPEED: f32 = 300.0;
    for (player, mut position) in &mut boxes {
        if trigger.client_id == **player {
            position.transform.translation +=
                Vec3::new(trigger.event.x, trigger.event.y, 0.0) * time.delta_secs() * MOVE_SPEED;
        }
    }
}
