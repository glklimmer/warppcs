use bevy::prelude::*;

use bevy_renet::{client_just_disconnected, netcode::NetcodeClientTransport};
use shared::GameState;

#[cfg(feature = "steam")]
use bevy::color::palettes::css::BURLYWOOD;
#[cfg(feature = "steam")]
use shared::steamworks::SteamworksClient;
#[cfg(feature = "steam")]
use steamworks::{LobbyId, SteamId};

#[cfg(feature = "steam")]
use crate::ui_widgets::text_input::TextInputValue;
use crate::ui_widgets::text_input::{
    TextInput, TextInputPlugin, TextInputTextColor, TextInputTextFont,
};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MainMenuStates {
    TitleScreen,
    Singleplayer,
    Multiplayer,
    JoinScreen,
    Lobby,
    None,
}

#[derive(Event, Clone)]
#[cfg(feature = "steam")]
pub struct JoinSteamLobby(pub SteamId);

#[derive(Component, PartialEq)]
enum Buttons {
    Singleplayer,
    Multiplayer,
    CreateLobby,
    JoinLobby,
    Join,
    #[allow(dead_code)]
    StartGame,
    #[allow(dead_code)]
    InvitePlayer,
    Back(MainMenuStates),
}

#[derive(Event, Clone)]
pub struct CleanMenuUI {}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TextInputPlugin);

        app.add_systems(OnEnter(MainMenuStates::TitleScreen), display_main_menu);
        app.add_systems(OnExit(MainMenuStates::TitleScreen), clean_ui);

        app.add_systems(
            OnEnter(MainMenuStates::Multiplayer),
            display_multiplayer_buttons,
        );
        app.add_systems(OnExit(MainMenuStates::Multiplayer), clean_ui);

        app.add_systems(OnEnter(MainMenuStates::JoinScreen), display_join_screen);
        app.add_systems(OnExit(MainMenuStates::JoinScreen), clean_ui);

        app.add_systems(Update, button_system);

        app.add_systems(OnEnter(GameState::GameSession), clean_ui);

        app.add_systems(Update, disconnect_client.run_if(client_just_disconnected));

        #[cfg(feature = "steam")]
        {
            app.add_event::<JoinSteamLobby>();
            app.add_systems(Update, change_state_on_button_steam);
        }

        app.add_systems(OnExit(MainMenuStates::Lobby), clean_ui);
        app.add_systems(OnExit(MainMenuStates::Lobby), disconect_client);
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn display_main_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Px(350.0),
                height: Val::Px(130.0),
                bottom: Val::Px(80.),
                left: Val::Px(40.),
                position_type: PositionType::Absolute,
                ..default()
            },
            Button,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(350.0),
                        height: Val::Px(65.0),
                        margin: UiRect::bottom(Val::Px(5.0)),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    BorderColor(Color::BLACK),
                    BackgroundColor(NORMAL_BUTTON),
                    Buttons::Singleplayer,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Singleplayer"),
                        TextFont {
                            font_size: 35.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(350.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    BorderColor(Color::BLACK),
                    BackgroundColor(NORMAL_BUTTON),
                    Buttons::Multiplayer,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Multiplayer"),
                        TextFont {
                            font_size: 35.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

fn clean_ui(mut commands: Commands, buttons_query: Query<Entity, With<Node>>) {
    for button_entity in buttons_query.iter() {
        commands.entity(button_entity).despawn_recursive();
    }
}

fn disconect_client(mut transport: ResMut<NetcodeClientTransport>) {
    transport.disconnect();
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Buttons>),
    >,
    asset_server: Res<AssetServer>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
                commands.spawn(AudioPlayer::<AudioSource>(
                    asset_server.load("sound/button_hover_sound.ogg"),
                ));
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
    mut button_query: Query<(&Interaction, &Buttons), Changed<Interaction>>,
    mut next_state: ResMut<NextState<MainMenuStates>>,
    mut player_commands: EventWriter<PlayerCommand>,
    mut join_lobby_request: EventWriter<JoinSteamLobby>,
    lobby_code: Query<&TextInputValue>,
    steam_client: Res<SteamworksClient>,
) {
    for (interaction, button) in &mut button_query {
        match *interaction {
            Interaction::Hovered => {}
            Interaction::Pressed => match button {
                Buttons::Singleplayer => next_state.set(MainMenuStates::Singleplayer),
                Buttons::Multiplayer => {
                    next_state.set(MainMenuStates::Multiplayer);
                }
                Buttons::CreateLobby => {
                    join_lobby_request.send(JoinSteamLobby(steam_client.user().steam_id()));
                }
                Buttons::JoinLobby => next_state.set(MainMenuStates::JoinScreen),
                Buttons::StartGame => {
                    player_commands.send(PlayerCommand::StartGame);
                }
                Buttons::InvitePlayer => steam_client
                    .friends()
                    .activate_invite_dialog(LobbyId::from_raw(76561198079103566)),
                Buttons::Join => match lobby_code.single().0.parse::<u64>() {
                    Ok(value) => {
                        join_lobby_request.send(JoinSteamLobby(SteamId::from_raw(value)));
                    }
                    Err(_) => {
                        println!("Invalid SteamID u64 value.")
                    }
                },
                Buttons::Back(state) => next_state.set(state.clone()),
            },
            Interaction::None => {}
        }
    }
}

fn display_multiplayer_buttons(mut commands: Commands) {
    commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: Val::Px(350.0),
            height: Val::Px(130.0),
            bottom: Val::Px(120.),
            left: Val::Px(40.),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(350.0),
                        height: Val::Px(65.0),
                        margin: UiRect::bottom(Val::Px(5.0)),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    BorderColor(Color::BLACK),
                    BackgroundColor(NORMAL_BUTTON),
                    Buttons::JoinLobby,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Join Lobby"),
                        TextFont {
                            font_size: 35.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(350.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    BorderColor(Color::BLACK),
                    BackgroundColor(NORMAL_BUTTON),
                    Buttons::CreateLobby,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Create Lobby"),
                        TextFont {
                            font_size: 35.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });

    commands
        .spawn((
            Node {
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
            Button,
            BorderColor(Color::BLACK),
            BackgroundColor(NORMAL_BUTTON),
            Buttons::Back(MainMenuStates::TitleScreen),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Back"),
                TextFont {
                    font_size: 35.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

const BORDER_COLOR_ACTIVE: Color = Color::srgb(0.75, 0.52, 0.99);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);

fn display_join_screen(mut commands: Commands) {
    commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(300.0),
                    border: UiRect::all(Val::Px(5.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
                TextInput,
                TextInputTextFont(TextFont {
                    font_size: 34.,
                    ..default()
                }),
                TextInputTextColor(TextColor(TEXT_COLOR)),
                BorderColor(BORDER_COLOR_ACTIVE),
                BackgroundColor(BACKGROUND_COLOR),
            ));
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    BorderColor(Color::BLACK),
                    BackgroundColor(NORMAL_BUTTON),
                    Buttons::Join,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Join"),
                        TextFont {
                            font_size: 35.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

fn disconnect_client(mut menu_state: ResMut<NextState<MainMenuStates>>) {
    println!("Disconnecting");
    menu_state.set(MainMenuStates::Multiplayer);
}
