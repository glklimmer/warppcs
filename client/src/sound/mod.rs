use animation_sound::AnimationSoundPlugin;
use background_sound::BackgroundSoundPlugin;
use bevy::prelude::*;

mod animation_sound;
mod background_sound;

pub const CRAFTING_SOUND_PATH: &str = "animation_sound/crafting";
pub const DIRT_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/dirt_footsteps";
pub const GRASS_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/grass_footsteps";
pub const SNOW_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/snow_footsteps";
pub const STONE_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/stone_footsteps";
pub const WATER_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/water_footsteps";
pub const HORSE_SOUND_PATH: &str = "animation_sound/horse";

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimationSoundPlugin);
        app.add_plugins(BackgroundSoundPlugin);
    }
}
