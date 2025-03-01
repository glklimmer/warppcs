use animation_sound::AnimationSoundPlugin;
use background_sound::BackgroundSoundPlugin;
use bevy::prelude::*;

mod animation_sound;
mod background_sound;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimationSoundPlugin);
        app.add_plugins(BackgroundSoundPlugin);
    }
}
