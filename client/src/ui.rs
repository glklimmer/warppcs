use bevy::prelude::*;
use shared::{networking::GoldAmount, GameState};

#[derive(Event)]
pub struct UpdateGoldAmount(pub GoldAmount);

#[derive(Component)]
pub struct GoldAmountDisplay;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateGoldAmount>();

        app.add_systems(OnEnter(GameState::GameSession), setup_ui);
    }
}

fn setup_ui(mut commands: Commands, mut gold_query: Query<&GoldAmount>) {
    let gold_amount = gold_query.single_mut();
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
                    format!("Gold Amount {:?}", gold_amount.0),
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

fn update_gold_amount(mut gold_query: Query<&GoldAmount>) {}
