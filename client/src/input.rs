use bevy::prelude::*;

use bevy_replicon::client::ClientSystems;
use shared::networking::LobbyMessage;

use crate::gizmos::GizmosSettings;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, lobby_input.before(ClientSystems::Send))
            .add_systems(
                Update,
                gizmos_settings.run_if(resource_changed::<ButtonInput<KeyCode>>),
            );
    }
}

fn lobby_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut lobby_events: MessageWriter<LobbyMessage>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        lobby_events.write(LobbyMessage::StartGame);
    }
}

fn gizmos_settings(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<GizmosSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyY) {
        settings.on = !settings.on;
    }
}
