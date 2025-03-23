use bevy::prelude::*;
use shared::{networking::ServerMessages, Faction, GameState};

use crate::networking::NetworkEvent;

#[derive(Component)]
pub struct GoldAmountDisplay;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameSession), setup_ui);

        app.add_systems(
            FixedUpdate,
            (update_gold_amount, display_player_defeat).run_if(in_state(GameState::GameSession)),
        );
    }
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: Val::Px(350.0),
            height: Val::Px(130.0),
            top: Val::Px(30.),
            right: Val::Px(0.),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("DISPLAY"),
                TextFont {
                    font_size: 25.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                GoldAmountDisplay,
            ));
        });
}

fn update_gold_amount(
    mut network_events: EventReader<NetworkEvent>,
    mut gold_display_query: Query<&mut Text, With<GoldAmountDisplay>>,
) {
    for event in network_events.read() {
        if let ServerMessages::SyncInventory(inventory) = &event.message {
            let mut gold_display = gold_display_query.single_mut();
            gold_display.0 = format!("Gold Amount {:?}", inventory.gold);
        }
    }
}

fn display_player_defeat(mut network_events: EventReader<NetworkEvent>) {
    for event in network_events.read() {
        if let ServerMessages::PlayerDefeat(player) = &event.message {
            match **player {
                Faction::Player(player) => println!("Player {} lost.", player),
                Faction::Bandits => todo!(),
            }
        }
    }
}
