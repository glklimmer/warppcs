use bevy::prelude::*;
use shared::networking::Inventory;

use crate::networking::ControlledPlayer;

#[derive(Component)]
pub struct GoldAmountDisplay;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, setup_ui);

        app.add_systems(FixedUpdate, update_gold_amount);
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
    mut gold_display_query: Query<&mut Text, With<GoldAmountDisplay>>,
    inventory_query: Query<&Inventory, With<ControlledPlayer>>,
) {
    if let Ok(inventory) = inventory_query.get_single() {
        let mut gold_display = gold_display_query.single_mut();
        gold_display.0 = format!("Gold Amount: {}", inventory.gold);
    }
}
