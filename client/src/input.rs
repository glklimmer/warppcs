use bevy::prelude::*;
use bevy_replicon::prelude::*;

use bevy_replicon::client::ClientSet;
use shared::networking::{PlayerCommand, PlayerInput, PlayerLobbyEvent};

use crate::gizmos::GizmosSettings;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // app.insert_resource(PlayerInput::default());
        // app.add_event::<PlayerCommand>();
        // app.add_systems(
        //     Update,
        //     (player_input, gizmos_settings).run_if(resource_changed::<ButtonInput<KeyCode>>),
        // );
        app.add_systems(
            PostUpdate,
            send_events.before(ClientSet::Send).run_if(client_connected),
        );
    }
}

fn send_events(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut lobby_events: EventWriter<PlayerLobbyEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        lobby_events.send(PlayerLobbyEvent::StartGame);
    }
}

fn player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_input: ResMut<PlayerInput>,
    mut player_commands: EventWriter<PlayerCommand>,
) {
    player_input.left =
        keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft);
    player_input.right =
        keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight);

    if keyboard_input.just_pressed(KeyCode::KeyE) {
        player_commands.send(PlayerCommand::MeleeAttack);
    }

    if keyboard_input.just_pressed(KeyCode::KeyF) {
        player_commands.send(PlayerCommand::Interact);
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
