use animations::sound::{ANIMATION_VOLUME, PlayAnimationSoundEvent};
use bevy::{audio::Volume, prelude::*};

use crate::ProjectileType;

pub(crate) struct ProjectileSoundPlugin;

impl Plugin for ProjectileSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(play_animation_on_projectile_spawn);
    }
}

fn play_animation_on_projectile_spawn(
    trigger: On<Add, ProjectileType>,
    mut projectile: Query<&ProjectileType>,
    mut sound_events: MessageWriter<PlayAnimationSoundEvent>,
    asset_server: Res<AssetServer>,
) -> Result {
    let projectile_type = projectile.get_mut(trigger.entity)?;

    let sound_handles = match projectile_type {
        ProjectileType::Arrow => vec![asset_server.load("animation_sound/arrow/arrow_flying.ogg")],
    };

    sound_events.write(PlayAnimationSoundEvent {
        entity: trigger.entity,
        sound_handles,
        speed: 1.5,
        volume: Volume::Linear(ANIMATION_VOLUME),
    });
    Ok(())
}
