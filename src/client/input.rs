use bevy::prelude::*;

use crate::shared::networking::{PlayerCommand, PlayerInput, UnitType};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerInput::default());
        app.add_event::<PlayerCommand>();
        app.add_systems(Update, player_input);
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

    if keyboard_input.just_pressed(KeyCode::Digit1) {
        player_commands.send(PlayerCommand::SpawnUnit(UnitType::Archer));
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        player_commands.send(PlayerCommand::SpawnUnit(UnitType::Shieldwarrior));
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        player_commands.send(PlayerCommand::SpawnUnit(UnitType::Pikeman));
    }
}
