use bevy::prelude::*;

use animations::sound::ANIMATION_VOLUME;
use bevy::audio::{PlaybackMode, Volume};
use bevy_replicon::prelude::{Channel, ServerEventAppExt};
use serde::{Deserialize, Serialize};

use crate::InteractionType;

pub(crate) struct InteractionSoundPlugin;

impl Plugin for InteractionSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(play_on_interactable)
            .add_server_event::<InteractableSound>(Channel::Ordered);
    }
}

#[derive(Event, Clone, Copy, Serialize, Deserialize)]
pub struct InteractableSound {
    pub kind: InteractionType,
    pub spatial_position: Vec3,
}

fn play_on_interactable(
    trigger: On<InteractableSound>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let audio_file = match trigger.kind {
        InteractionType::Recruit => {
            asset_server.load("animation_sound/recruitment/recruite_call.ogg")
        }
        InteractionType::Flag => todo!(),
        InteractionType::Building => todo!(),
        InteractionType::Mount => todo!(),
        InteractionType::Travel => todo!(),
        InteractionType::Chest => todo!(),
        InteractionType::Item => todo!(),
        InteractionType::Commander => todo!(),
        InteractionType::ItemAssignment => todo!(),
        InteractionType::Unmount => todo!(),
        InteractionType::Portal => todo!(),
    };

    commands.spawn((
        AudioPlayer::<AudioSource>(audio_file),
        PlaybackSettings {
            mode: PlaybackMode::Remove,
            volume: Volume::Linear(ANIMATION_VOLUME * 0.5),
            spatial: true,
            ..default()
        },
        Transform::from_translation(trigger.spatial_position),
    ));
}
