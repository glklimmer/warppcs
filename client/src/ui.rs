use bevy::{color::palettes::css::BURLYWOOD, prelude::*};
use bevy_renet::renet::ClientId;
use shared::networking::{MultiplayerRoles, PlayerCommand};

#[cfg(feature = "steam")]
use shared::steamworks::SteamworksClient;
#[cfg(feature = "steam")]
use steamworks::{LobbyId, SteamId};

use crate::{
    networking::{CurrentClientId, PlayerJoined},
    ui_widgets::text_input::{
        TextInputBundle, TextInputPlugin, TextInputSubmitEvent, TextInputSystem, TextInputValue,
    },
};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MainMenuStates {
    TitleScreen,
    Singleplayer,
    Multiplayer,
    JoinScreen,
    Lobby,
}

#[cfg(feature = "netcode")]
use std::net::SocketAddr;

#[derive(Event, Clone)]
#[cfg(feature = "steam")]
pub struct JoinSteamLobby(pub SteamId);

#[derive(Event, Clone)]
#[cfg(feature = "netcode")]
pub struct JoinNetcodeLobby(pub SocketAddr);

#[derive(Component, PartialEq)]
enum Button {
    Singleplayer,
    Multiplayer,
    CreateLobby,
    JoinLobby,
    Join,
    StartGame,
    InvitePlayer,
    Back(MainMenuStates),
}

#[derive(Component)]
enum Checkbox {
    Checked,
    Unchecked,
}

#[derive(Component)]
struct LobbySlotOwner(ClientId);

#[derive(Component)]
struct LobbySlotName;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerJoined>();

        app.add_plugins(TextInputPlugin);

        app.add_systems(OnEnter(MainMenuStates::TitleScreen), display_main_menu);

        app.add_systems(
            OnEnter(MainMenuStates::Multiplayer),
            display_multiplayer_buttons,
        );

        app.add_systems(OnEnter(MainMenuStates::JoinScreen), display_join_screen);

        app.add_systems(
            Update,
            (lobby_slot_checkbox, add_player_to_lobby_slot).run_if(in_state(MainMenuStates::Lobby)),
        );

        app.add_systems(
            Update,
            listener
                .after(TextInputSystem)
                .run_if(in_state(MainMenuStates::Lobby)),
        );

        app.add_systems(Update, button_system);

        #[cfg(feature = "steam")]
        {
            app.add_event::<JoinSteamLobby>();

            app.add_systems(OnEnter(MainMenuStates::Lobby), display_steam_lobby);

            app.add_systems(Update, change_state_on_button_steam);
        }
        #[cfg(feature = "netcode")]
        {
            app.add_event::<JoinNetcodeLobby>();

            app.add_systems(OnEnter(MainMenuStates::Lobby), display_netcode_lobby);

            app.add_systems(Update, change_state_on_button_netcode);
        }
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn display_main_menu(mut commands: Commands) {
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
                    Button::Singleplayer,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Singleplayer",
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
                    Button::Multiplayer,
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

#[allow(clippy::type_complexity)]
fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Button,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    buttons_query: Query<Entity, With<Node>>,
    asset_server: Res<AssetServer>,
) {
    for (interaction, mut color, mut border_color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if *button != Button::InvitePlayer {
                    for button_entity in buttons_query.iter() {
                        commands.entity(button_entity).despawn_recursive();
                    }
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

#[cfg(feature = "steam")]
fn change_state_on_button_steam(
    mut button_query: Query<(&Interaction, &Button), Changed<Interaction>>,
    mut next_state: ResMut<NextState<MainMenuStates>>,
    mut multiplayer_roles: ResMut<NextState<MultiplayerRoles>>,
    mut player_commands: EventWriter<PlayerCommand>,
    mut join_lobby_request: EventWriter<JoinSteamLobby>,
    lobby_code: Query<&TextInputValue>,
    steam_client: Res<SteamworksClient>,
) {
    for (interaction, button) in &mut button_query {
        match *interaction {
            Interaction::Hovered => {}
            Interaction::Pressed => match button {
                Button::Singleplayer => next_state.set(MainMenuStates::Singleplayer),
                Button::Multiplayer => next_state.set(MainMenuStates::Multiplayer),
                Button::CreateLobby => multiplayer_roles.set(MultiplayerRoles::Host),
                Button::JoinLobby => next_state.set(MainMenuStates::JoinScreen),
                Button::StartGame => {
                    // TODO
                    player_commands.send(PlayerCommand::StartGame);
                }
                Button::InvitePlayer => steam_client
                    .friends()
                    .activate_invite_dialog(LobbyId::from_raw(76561198079103566)),
                Button::Join => match lobby_code.single().0.parse::<u64>() {
                    Ok(value) => {
                        join_lobby_request.send(JoinSteamLobby(SteamId::from_raw(value)));
                        multiplayer_roles.set(MultiplayerRoles::Client);
                    }
                    Err(_) => {
                        println!("Invalid u64 value.")
                    }
                },
                Button::Back(state) => next_state.set(state.clone()),
            },
            Interaction::None => {}
        }
    }
}

#[cfg(feature = "netcode")]
fn change_state_on_button_netcode(
    mut button_query: Query<(&Interaction, &Button), Changed<Interaction>>,
    mut next_state: ResMut<NextState<MainMenuStates>>,
    mut multiplayer_roles: ResMut<NextState<MultiplayerRoles>>,
    mut player_commands: EventWriter<PlayerCommand>,
) {
    for (interaction, button) in &mut button_query {
        match *interaction {
            Interaction::Hovered => {}
            Interaction::Pressed => match button {
                Button::Singleplayer => next_state.set(MainMenuStates::Singleplayer),
                Button::Multiplayer => next_state.set(MainMenuStates::Multiplayer),
                Button::CreateLobby => multiplayer_roles.set(MultiplayerRoles::Host),
                Button::JoinLobby => next_state.set(MainMenuStates::JoinScreen),
                Button::StartGame => {
                    // TODO
                    player_commands.send(PlayerCommand::StartGame);
                }
                Button::InvitePlayer => todo!(),
                Button::Join => todo!(),
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
            Button::Back(MainMenuStates::TitleScreen),
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

const BORDER_COLOR_ACTIVE: Color = Color::srgb(0.75, 0.52, 0.99);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);

fn display_join_screen(mut commands: Commands) {
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
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        border: UiRect::all(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                    border_color: BORDER_COLOR_ACTIVE.into(),
                    background_color: BACKGROUND_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default().with_text_style(TextStyle {
                    font_size: 40.,
                    color: TEXT_COLOR,
                    ..default()
                }),
            ));
        })
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
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
                    Button::Join,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Join",
                        TextStyle {
                            font_size: 35.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}

fn listener(mut events: EventReader<TextInputSubmitEvent>) {
    for event in events.read() {
        info!("{:?} submitted: {}", event.entity, event.value);
    }
}

#[cfg(feature = "steam")]
fn display_steam_lobby(
    mut commands: Commands,
    steam_client: Res<SteamworksClient>,
    asset_server: Res<AssetServer>,
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
                        width: Val::Px(400.0),
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
                    let client_id = steam_client.user().steam_id().raw();
                    parent.spawn(TextBundle::from_section(
                        format!("Code: {client_id}"),
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
                                Button::InvitePlayer,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle::from_section(
                                        format!("Slot {i}"),
                                        TextStyle {
                                            font_size: 35.0,
                                            color: Color::srgb(0.9, 0.9, 0.9),
                                            ..default()
                                        },
                                    ),
                                    LobbySlotName,
                                ));
                            });
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(50.),
                                    height: Val::Px(50.),
                                    ..Default::default()
                                },
                                visibility: Visibility::Hidden,
                                image: UiImage::new(asset_server.load("ui/checkbox.png")),
                                ..Default::default()
                            },
                            Checkbox::Unchecked,
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
            Button::Back(MainMenuStates::Multiplayer),
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

#[cfg(feature = "netcode")]
fn display_netcode_lobby(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            parent.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(400.0),
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
            });
        })
        .with_children(|parent| {
            for i in 1..5 {
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
                                Button::InvitePlayer,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle::from_section(
                                        format!("Slot {i}"),
                                        TextStyle {
                                            font_size: 35.0,
                                            color: Color::srgb(0.9, 0.9, 0.9),
                                            ..default()
                                        },
                                    ),
                                    LobbySlotName,
                                ));
                            });
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(50.),
                                    height: Val::Px(50.),
                                    ..Default::default()
                                },
                                visibility: Visibility::Hidden,
                                image: UiImage::new(asset_server.load("ui/checkbox.png")),
                                ..Default::default()
                            },
                            Checkbox::Unchecked,
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
            Button::Back(MainMenuStates::Multiplayer),
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

#[allow(clippy::type_complexity)]
fn add_player_to_lobby_slot(
    mut commands: Commands,
    mut text_query: Query<
        (Entity, &mut Text, &Parent),
        (Without<LobbySlotOwner>, With<LobbySlotName>),
    >,
    mut checkbox_query: Query<&mut Visibility, With<Checkbox>>,
    mut player_joined: EventReader<PlayerJoined>,
    slots: Query<&Children>,
) {
    for event in player_joined.read() {
        if let Some((entity, mut text, parent)) = (&mut text_query).into_iter().next() {
            text.sections[0].value = event.0.to_string();
            commands.entity(entity).insert(LobbySlotOwner(event.0));
            for child in slots.get(parent.get()).unwrap().iter() {
                if let Ok(mut checkbox) = checkbox_query.get_mut(*child) {
                    *checkbox = Visibility::Visible;
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
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
        if lobby_slot_owner.0.raw().eq(&client_id.0) {
            continue;
        }
        match *interactions {
            Interaction::Pressed => match *checkbox {
                Checkbox::Checked => {
                    *checkbox_image = UiImage::new(asset_server.load("ui/checkbox.png"));
                    *checkbox = Checkbox::Unchecked;
                    commands.spawn(AudioBundle {
                        source: asset_server.load("sound/switch_002.ogg"),
                        ..Default::default()
                    });
                }
                Checkbox::Unchecked => {
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
