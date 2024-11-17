use bevy::prelude::*;
use shared::{networking::ServerMessages, GameState};

use crate::networking::{Connected, NetworkEvent};

#[derive(Component)]
pub struct GoldAmountDisplay;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameSession), setup_ui);

        app.add_systems(
            FixedUpdate,
            update_gold_amount
                .run_if(on_event::<NetworkEvent>())
                .in_set(Connected),
        );
    }
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Px(350.0),
                height: Val::Px(130.0),
                top: Val::Px(30.),
                right: Val::Px(0.),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "GOLD DISPLAY".to_string(),
                    TextStyle {
                        font_size: 25.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
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
            gold_display.sections[0].value = format!("Gold Amount {:?}", inventory.gold);
        }
    }
}
