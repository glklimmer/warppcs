use bevy::{input::mouse::mouse_button_input_system, prelude::*};

pub struct MenuPlugin;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    MainMenu,
    SinglePlayer,
    MultiPlayer,
    JoinLobby,
    CreateLooby,
}

#[derive(Component)]
enum Button {
    SinglePlayer,
    MultiPlayer,
    CreateLobby,
    JoinLobby,
}
#[derive(Component)]
enum Checkbox {
    Checked,
    None,
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(AppState::MainMenu);

        app.add_systems(Startup, setup);

        app.add_systems(OnEnter(AppState::MultiPlayer), display_multiplayer_buttons);

        app.add_systems(OnEnter(AppState::CreateLooby), display_create_lobby);
        app.add_systems(Update, checkbox.run_if(in_state(AppState::CreateLooby)));

        app.add_systems(Update, (button_system, change_state_on_button));
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(350.0),
                            height: Val::Px(65.0),
                            margin: UiRect::bottom(Val::Px(5.0)),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    Button::SinglePlayer,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Single Player",
                        TextStyle {
                            font_size: 35.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        })
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(350.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    Button::MultiPlayer,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Multiplayer",
                        TextStyle {
                            font_size: 35.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}

fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    buttons_query: Query<Entity, With<Button>>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                for button_entity in buttons_query.iter() {
                    commands.entity(button_entity).despawn_recursive();
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn change_state_on_button(
    mut button_query: Query<(&Interaction, &Button), (Changed<Interaction>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, button) in &mut button_query {
        match *interaction {
            Interaction::Hovered => {}
            Interaction::Pressed => match button {
                Button::SinglePlayer => next_state.set(AppState::SinglePlayer),
                Button::MultiPlayer => next_state.set(AppState::MultiPlayer),
                Button::CreateLobby => next_state.set(AppState::CreateLooby),
                Button::JoinLobby => next_state.set(AppState::JoinLobby),
            },
            Interaction::None => {}
        }
    }
}

fn display_multiplayer_buttons(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(350.0),
                            height: Val::Px(65.0),
                            margin: UiRect::bottom(Val::Px(5.0)),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    Button::JoinLobby,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Join Lobby",
                        TextStyle {
                            font_size: 35.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        })
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(350.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    Button::CreateLobby,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Create Lobby",
                        TextStyle {
                            font_size: 35.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}

fn display_create_lobby(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for i in 1..5 {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            width: Val::Percent(50.0),
                            height: Val::Percent(20.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            border: UiRect::all(Val::Px(2.0)),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            format!("Player {i}"),
                            TextStyle {
                                font_size: 35.0,
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ));
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(50.),
                                    height: Val::Px(50.),

                                    ..Default::default()
                                },
                                image: UiImage::new(asset_server.load("ui/checkbox.png")),

                                ..Default::default()
                            },
                            Checkbox::None,
                        ));
                    });
            }
        });
}

fn checkbox(
    mut checkbox_query: Query<
        (&Interaction, &mut UiImage, &mut Checkbox),
        (Changed<Interaction>, With<Checkbox>),
    >,
    asset_server: Res<AssetServer>,
) {
    for (interactions, mut checkbox_image, mut checkbox) in &mut checkbox_query {
        match *interactions {
            Interaction::Pressed => {
                match *checkbox {
                    Checkbox::Checked => {
                        *checkbox_image = UiImage::new(asset_server.load("ui/checkbox.png"));
                        *checkbox = Checkbox::None;
                    }
                    Checkbox::None => {
                        *checkbox_image =
                            UiImage::new(asset_server.load("ui/checkbox_checked.png"));
                        *checkbox = Checkbox::Checked;
                    }
                }

                println!("clicked")
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
