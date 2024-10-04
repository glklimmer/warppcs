use bevy::{color::palettes::css::BURLYWOOD, prelude::*};

use crate::networking::CurrentClientId;

pub struct MenuPlugin;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    MainMenu,
    SinglePlayer,
    MultiPlayer,
    JoinLobby,
    CreateLooby,
    GameSession,
}

#[derive(Component)]
enum Button {
    SinglePlayer,
    MultiPlayer,
    CreateLobby,
    JoinLobby,
    StartGame,
    Back(AppState),
}

#[derive(Component)]
enum Checkbox {
    Checked,
    None,
}

#[derive(Component)]
struct LobbySlotOwner(u64);

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(AppState::MainMenu);

        app.add_systems(Startup, setup);

        app.add_systems(OnEnter(AppState::MainMenu), setup);

        app.add_systems(OnEnter(AppState::MultiPlayer), display_multiplayer_buttons);

        app.add_systems(OnEnter(AppState::CreateLooby), display_create_lobby);
        app.add_systems(
            Update,
            lobby_slot_checkbox.run_if(in_state(AppState::CreateLooby)),
        );

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
                width: Val::Px(350.0),
                height: Val::Px(130.0),
                bottom: Val::Px(80.),
                left: Val::Px(40.),
                position_type: PositionType::Absolute,
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
                            justify_content: JustifyContent::Center,
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
                            justify_content: JustifyContent::Center,
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
    buttons_query: Query<Entity, With<Node>>,
    asset_server: Res<AssetServer>,
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
                commands.spawn(AudioBundle {
                    source: asset_server.load("sound/button_hover_sound.ogg"),
                    ..Default::default()
                });
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
                Button::StartGame => next_state.set(AppState::GameSession),
                Button::Back(state) => next_state.set(state.clone()),
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
                width: Val::Px(350.0),
                height: Val::Px(130.0),
                bottom: Val::Px(120.),
                left: Val::Px(40.),
                position_type: PositionType::Absolute,
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
                            justify_content: JustifyContent::Center,
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
                            justify_content: JustifyContent::Center,
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

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(50.0),
                    border: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Start,
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    bottom: Val::Percent(5.),
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Button::Back(AppState::MainMenu),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Back",
                TextStyle {
                    font_size: 30.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        });
}

fn display_create_lobby(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    client_id: Res<CurrentClientId>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(50.0),
                height: Val::Percent(80.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                top: Val::Percent(5.),
                ..Default::default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((ButtonBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        margin: UiRect::bottom(Val::Px(5.0)),
                        right: Val::Px(0.),
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Code",
                        TextStyle {
                            font_size: 35.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        })
        .with_children(|parent| {
            for i in 1..5 {
                let mut lobby_slot_owner = 0;
                if i == 1 {
                    lobby_slot_owner = client_id.0
                }
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            width: Val::Percent(80.0),
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
                            LobbySlotOwner(lobby_slot_owner),
                            Checkbox::None,
                        ));
                    });
            }
        })
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Percent(40.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::top(Val::Px(5.0)),
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: BURLYWOOD.into(),
                        ..default()
                    },
                    Button::StartGame,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Start Game",
                        TextStyle {
                            font_size: 35.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });

    commands.spawn(NodeBundle {
        style: Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            width: Val::Percent(50.0),
            height: Val::Percent(80.0),
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            border: UiRect::all(Val::Px(2.)),
            right: Val::Px(0.),
            ..Default::default()
        },
        ..default()
    });

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(50.0),
                    border: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Start,
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    bottom: Val::Percent(5.),
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Button::Back(AppState::MultiPlayer),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Back",
                TextStyle {
                    font_size: 30.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        });
}

fn lobby_slot_checkbox(
    mut commands: Commands,
    mut checkbox_query: Query<
        (&Interaction, &mut UiImage, &mut Checkbox, &LobbySlotOwner),
        (Changed<Interaction>, With<Checkbox>),
    >,
    asset_server: Res<AssetServer>,
    client_id: Res<CurrentClientId>,
) {
    for (interactions, mut checkbox_image, mut checkbox, lobby_slot_owner) in &mut checkbox_query {
        if lobby_slot_owner.0 != client_id.0 {
            continue;
        }
        match *interactions {
            Interaction::Pressed => match *checkbox {
                Checkbox::Checked => {
                    *checkbox_image = UiImage::new(asset_server.load("ui/checkbox.png"));
                    *checkbox = Checkbox::None;
                    commands.spawn(AudioBundle {
                        source: asset_server.load("sound/switch_002.ogg"),
                        ..Default::default()
                    });
                }
                Checkbox::None => {
                    *checkbox_image = UiImage::new(asset_server.load("ui/checkbox_checked.png"));
                    *checkbox = Checkbox::Checked;
                    commands.spawn(AudioBundle {
                        source: asset_server.load("sound/switch_002.ogg"),
                        ..Default::default()
                    });
                }
            },
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
