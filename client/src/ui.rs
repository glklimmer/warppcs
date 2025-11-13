use bevy::prelude::*;
use shared::{ControlledPlayer, GameState, networking::Inventory};

#[derive(Component)]
pub struct GoldAmountDisplay;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, setup_ui);

        app.add_systems(
            FixedUpdate,
            update_gold_amount.run_if(in_state(GameState::GameSession)),
        );
    }
}

fn setup_ui(mut commands: Commands) -> Result {
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
    Ok(())
}

fn update_gold_amount(
    mut gold_display_query: Query<&mut Text, With<GoldAmountDisplay>>,
    inventory_query: Query<&Inventory, With<ControlledPlayer>>,
) -> Result {
    let inventory = inventory_query.single()?;
    let mut gold_display = gold_display_query.single_mut()?;
    gold_display.0 = format!("Gold Amount: {}", inventory.gold);
    Ok(())
}
