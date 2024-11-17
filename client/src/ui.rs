use bevy::prelude::*;
use shared::{server::economy::GoldAmount, GameState};

#[derive(Event)]
pub struct UpdateGoldAmount {
    pub gold_amount: GoldAmount,
}

#[derive(Component)]
pub struct GoldAmountDisplay;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateGoldAmount>();

        app.add_systems(OnEnter(GameState::GameSession), setup_ui);

        app.add_systems(
            Update,
            update_gold_amount.run_if(on_event::<UpdateGoldAmount>()),
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
                    format!("Gold Amount: 0"),
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
    mut gold_event: EventReader<UpdateGoldAmount>,
    mut gold_display_query: Query<&mut Text, With<GoldAmountDisplay>>,
) {
    for event in gold_event.read() {
        let mut gold_display = gold_display_query.single_mut();
        gold_display.sections[0].value = format!("Gold Amount {:?}", event.gold_amount.0);
    }
}

fn disconnect_client(
    mut menu_state: ResMut<NextState<MainMenuStates>>,
    mut multiplayer_roles: ResMut<NextState<MultiplayerRoles>>,
) {
    println!("Disconnecting");
    menu_state.set(MainMenuStates::Multiplayer);
    multiplayer_roles.set(MultiplayerRoles::NotInGame);
}
