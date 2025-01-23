use bevy::prelude::*;

use bevy_renet::{
    client_just_disconnected,
    netcode::NetcodeClientTransport,
    renet::{ClientId, RenetClient},
};
use shared::{
    networking::{Checkbox, ClientChannel, MultiplayerRoles, PlayerCommand, ServerMessages},
    GameState,
};

#[cfg(feature = "steam")]
use bevy::color::palettes::css::BURLYWOOD;
#[cfg(feature = "steam")]
use shared::steamworks::SteamworksClient;
#[cfg(feature = "steam")]
use steamworks::{LobbyId, SteamId};

#[cfg(feature = "steam")]
use crate::ui_widgets::text_input::TextInputValue;
use crate::{
    networking::{CurrentClientId, NetworkEvent},
    ui_widgets::text_input::{TextInput, TextInputPlugin, TextInputTextColor, TextInputTextFont},
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

#[derive(Component)]
struct LobbySlotOwner(ClientId);

#[derive(Component)]
struct LobbySlotName(u8);

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

        app.add_systems(
            FixedUpdate,
            (
                add_player_to_lobby_slot,
                update_players_checkbox,
                remove_player_from_lobby,
            )
                .run_if(on_event::<NetworkEvent>),
        );

        app.add_systems(Update, button_system);

        app.add_systems(OnEnter(GameState::GameSession), clean_ui);

        app.add_systems(Update, disconnect_client.run_if(client_just_disconnected));

        #[cfg(feature = "steam")]
        {
            app.add_event::<JoinSteamLobby>();
            app.add_systems(OnEnter(MainMenuStates::Lobby), display_steam_lobby);
            app.add_systems(Update, change_state_on_button_steam);
        }

        app.add_systems(OnExit(MainMenuStates::Lobby), clean_ui);
        app.add_systems(OnExit(MainMenuStates::Lobby), disconect_client);

        app.add_systems(
            Update,
            (lobby_slot_checkbox).run_if(in_state(MainMenuStates::Lobby)),
        );
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
                Buttons::Singleplayer => next_state.set(MainMenuStates::Singleplayer),
                Buttons::Multiplayer => {
                    next_state.set(MainMenuStates::Multiplayer);
                }
                Buttons::CreateLobby => multiplayer_roles.set(MultiplayerRoles::Host),
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
                        multiplayer_roles.set(MultiplayerRoles::Client);
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

#[cfg(feature = "steam")]
fn display_steam_lobby(
    mut commands: Commands,
    steam_client: Res<SteamworksClient>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: Val::Percent(50.0),
            height: Val::Percent(80.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            top: Val::Percent(5.),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        margin: UiRect::bottom(Val::Px(5.0)),
                        right: Val::Px(0.),
                        ..default()
                    },
                    Button,
                    BorderColor(Color::BLACK),
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_children(|parent| {
                    let client_id = steam_client.user().steam_id().raw();
                    parent.spawn((
                        Text::new(format!("Code: {client_id}")),
                        TextFont {
                            font_size: 35.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        })
        .with_children(|parent| {
            for i in 1..5 {
                parent
                    .spawn((
                        Node {
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
                        BorderColor(Color::BLACK),
                    ))
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
                                BorderColor(Color::BLACK),
                                BackgroundColor(NORMAL_BUTTON),
                                Buttons::InvitePlayer,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(format!("Slot {i}")),
                                    TextFont {
                                        font_size: 35.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                    LobbySlotName(i),
                                ));
                            });
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            Node {
                                width: Val::Px(50.),
                                height: Val::Px(50.),
                                ..default()
                            },
                            Button,
                            Visibility::Hidden,
                            ImageNode::new(asset_server.load("ui/checkbox.png")),
                            LobbySlotName(i),
                            Checkbox::Unchecked,
                        ));
                    });
            }
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(40.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::top(Val::Px(5.0)),
                        ..default()
                    },
                    Button,
                    BorderColor(Color::BLACK),
                    BackgroundColor(BURLYWOOD.into()),
                    Buttons::StartGame,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Start Game"),
                        TextFont {
                            font_size: 35.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });

    commands.spawn(Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        width: Val::Percent(50.0),
        height: Val::Percent(80.0),
        align_items: AlignItems::Center,
        position_type: PositionType::Absolute,
        border: UiRect::all(Val::Px(2.)),
        right: Val::Px(0.),
        ..Default::default()
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
            Buttons::Back(MainMenuStates::Multiplayer),
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

#[allow(clippy::type_complexity)]
fn add_player_to_lobby_slot(
    mut commands: Commands,
    mut text_query: Query<(Entity, &mut Text, &LobbySlotName), Without<LobbySlotOwner>>,
    mut checkbox_query: Query<
        (Entity, &mut Visibility, &LobbySlotName, &mut ImageNode),
        (With<Checkbox>, Without<LobbySlotOwner>),
    >,
    mut network_events: EventReader<NetworkEvent>,
    asset_server: Res<AssetServer>,
) {
    let mut text_query_sorted = (&mut text_query)
        .into_iter()
        .sort_by_key::<&LobbySlotName, _>(|slot| slot.0);

    let mut checkbox_query_sorted = (&mut checkbox_query)
        .into_iter()
        .sort_by_key::<&LobbySlotName, _>(|slot| slot.0);

    for event in network_events.read() {
        if let ServerMessages::PlayerJoinedLobby { id, ready_state } = &event.message {
            if let Some((entity, mut text, _)) = text_query_sorted.next() {
                text.0 = id.to_string();
                commands.entity(entity).insert(LobbySlotOwner(*id));
            }
            if let Some((entity, mut checkbox, _, mut checkbox_image)) =
                checkbox_query_sorted.next()
            {
                *checkbox = Visibility::Visible;
                commands.entity(entity).insert(LobbySlotOwner(*id));
                match ready_state {
                    Checkbox::Checked => {
                        *checkbox_image =
                            ImageNode::new(asset_server.load("ui/checkbox_checked.png"));
                    }
                    Checkbox::Unchecked => {
                        *checkbox_image = ImageNode::new(asset_server.load("ui/checkbox.png"));
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn remove_player_from_lobby(
    mut commands: Commands,
    mut text_query: Query<
        (Entity, &mut Text, &LobbySlotOwner, &LobbySlotName),
        With<LobbySlotOwner>,
    >,
    mut checkbox_query: Query<
        (
            Entity,
            &mut Visibility,
            &mut ImageNode,
            &mut Checkbox,
            &LobbySlotOwner,
        ),
        (With<Checkbox>, With<LobbySlotOwner>),
    >,
    asset_server: Res<AssetServer>,
    mut network_events: EventReader<NetworkEvent>,
) {
    for event in network_events.read() {
        if let ServerMessages::PlayerLeftLobby { id } = &event.message {
            for (entity, mut text, lobby, slot) in &mut text_query {
                if lobby.0.eq(id) {
                    commands.entity(entity).remove::<LobbySlotOwner>();
                    text.0 = format!("Slot {}", slot.0);
                }
            }
            for (entity, mut visibility, mut checkbox_image, mut checkbox, lobb) in
                &mut checkbox_query
            {
                if lobb.0.eq(id) {
                    commands.entity(entity).remove::<LobbySlotOwner>();
                    *visibility = Visibility::Hidden;
                    *checkbox_image = ImageNode::new(asset_server.load("ui/checkbox.png"));
                    *checkbox = Checkbox::Unchecked;
                }
            }
        }
    }
}

fn update_players_checkbox(
    mut network_events: EventReader<NetworkEvent>,
    mut checkbox_query: Query<(&LobbySlotOwner, &mut ImageNode), With<Checkbox>>,
    asset_server: Res<AssetServer>,
) {
    for event in network_events.read() {
        if let ServerMessages::LobbyPlayerReadyState { id, ready_state } = &event.message {
            for (lobby_slot_owner, mut checkbox_image) in &mut checkbox_query {
                if lobby_slot_owner.0.eq(id) {
                    match ready_state {
                        Checkbox::Checked => {
                            *checkbox_image =
                                ImageNode::new(asset_server.load("ui/checkbox_checked.png"));
                        }
                        Checkbox::Unchecked => {
                            *checkbox_image = ImageNode::new(asset_server.load("ui/checkbox.png"));
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn lobby_slot_checkbox(
    mut commands: Commands,
    mut checkbox_query: Query<
        (&Interaction, &mut ImageNode, &mut Checkbox, &LobbySlotOwner),
        (Changed<Interaction>, With<Checkbox>),
    >,
    mut client: ResMut<RenetClient>,
    asset_server: Res<AssetServer>,
    client_id: Res<CurrentClientId>,
) {
    for (interactions, mut checkbox_image, mut checkbox, lobby_slot_owner) in &mut checkbox_query {
        if !lobby_slot_owner.0.eq(&client_id.0) {
            continue;
        }
        match *interactions {
            Interaction::Pressed => match *checkbox {
                Checkbox::Checked => {
                    *checkbox_image = ImageNode::new(asset_server.load("ui/checkbox.png"));
                    *checkbox = Checkbox::Unchecked;
                    commands.spawn(AudioPlayer::<AudioSource>(
                        asset_server.load("sound/switch_002.ogg"),
                    ));

                    let message = PlayerCommand::LobbyReadyState(Checkbox::Unchecked);

                    let command = bincode::serialize(&message).unwrap();

                    client.send_message(ClientChannel::Command, command);
                }

                Checkbox::Unchecked => {
                    *checkbox_image = ImageNode::new(asset_server.load("ui/checkbox_checked.png"));
                    *checkbox = Checkbox::Checked;
                    commands.spawn(AudioPlayer::<AudioSource>(
                        asset_server.load("sound/switch_002.ogg"),
                    ));

                    let message = PlayerCommand::LobbyReadyState(Checkbox::Checked);

                    let command = bincode::serialize(&message).unwrap();

                    client.send_message(ClientChannel::Command, command);
                }
            },
            Interaction::Hovered => {}
            Interaction::None => {}
        }
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
