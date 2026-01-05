use bevy::prelude::*;

use bevy_replicon::prelude::{SendMode, ToClients};
use health::{DelayedDespawn, Health};
use shared::{AnimationChange, AnimationChangeEvent};

pub(crate) struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_unit_death);
    }
}

fn on_unit_death(
    death: On<Remove, Health>,
    mut unit_animation: MessageWriter<ToClients<AnimationChangeEvent>>,
    mut commands: Commands,
) -> Result {
    let entity = death.entity;

    commands
        .entity(entity)
        .insert(DelayedDespawn(Timer::from_seconds(600., TimerMode::Once)));

    unit_animation.write(ToClients {
        mode: SendMode::Broadcast,
        message: AnimationChangeEvent {
            entity,
            change: AnimationChange::Death,
        },
    });

    Ok(())
}
