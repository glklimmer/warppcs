use bevy::prelude::*;

use bevy::color::palettes::css::{BLUE, RED};
use bevy_renet::renet::{ClientId, RenetServer};
use std::env;

use crate::map::scenes::base::BaseScene;
use crate::map::scenes::fight::FightScene;
use crate::map::base::{BaseScene, RecruitmentBuilding};
use crate::map::{GameScene, GameSceneType, Layers};
use crate::networking::{
    Inventory, Owner, PlayerInput, ServerChannel, ServerMessages, SpawnPlayer,
};
use crate::server::lobby::GameLobby;
use crate::server::networking::ServerLobby;
use crate::server::physics::movement::Velocity;
use crate::GameState;
use crate::{
    map::GameSceneId,
    networking::{PlayerCommand, PlayerSkin},
    server::{
        game_scenes::GameSceneDestination,
        networking::{GameWorld, NetworkEvent},
    },
};

pub struct StartGamePlugin;

impl Plugin for StartGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (start_game).run_if(on_event::<NetworkEvent>()));
    }
}

#[allow(clippy::too_many_arguments)]
fn start_game(
    mut commands: Commands,
    mut network_events: EventReader<NetworkEvent>,
    mut game_world: ResMut<GameWorld>,
    mut server: ResMut<RenetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    lobby: Res<ServerLobby>,
    game_lobby: Res<GameLobby>,
) {
    for event in network_events.read() {
        if let PlayerCommand::StartGame = &event.message {
            if !game_lobby.all_ready() {
                continue;
            }
            println!("Starting game...");
            let args: Vec<String> = env::args().collect();
            if args.contains(&String::from("fight")) {
                fight_map(&lobby, &mut commands, &mut server);
            } else {
                duel_map(&lobby, &mut game_world, &mut commands, &mut server);
            }

            next_state.set(GameState::GameSession);
        }
    }
}

fn fight_map(lobby: &Res<ServerLobby>, commands: &mut Commands, server: &mut ResMut<RenetServer>) {
    let mut players: Vec<(&ClientId, &Entity)> = lobby.players.iter().collect();
    let (left_client_id, left_player_entity) = players.pop().unwrap();
    let (right_client_id, right_player_entity) = players.pop().unwrap();

    // Create Fight Scene
    let base = FightScene::new();
    let server_components = (Owner(*left_client_id), GameSceneId(1));
    commands.spawn((base.left_main_building, server_components));
    commands.spawn((base.left_archer_building, server_components));
    commands.spawn((base.left_warrior_building, server_components));
    commands.spawn((base.left_pikeman_building, server_components));
    commands.spawn((base.left_left_wall, server_components));
    commands.spawn((base.left_right_wall, server_components));
    commands.spawn((base.left_gold_farm, server_components));

    let server_components = (Owner(*right_client_id), GameSceneId(1));
    commands.spawn((base.right_main_building, server_components));
    commands.spawn((base.right_archer_building, server_components));
    commands.spawn((base.right_warrior_building, server_components));
    commands.spawn((base.right_pikeman_building, server_components));
    commands.spawn((base.right_left_wall, server_components));
    commands.spawn((base.right_right_wall, server_components));
    commands.spawn((base.right_gold_farm, server_components));

    // Create Player entities
    let left_transform = Transform::from_xyz(-1500., 50., Layers::Player.as_f32());
    let inventory = Inventory { gold: 100 };
    commands.entity(*left_player_entity).insert((
        left_transform,
        PlayerInput::default(),
        Velocity::default(),
        GameSceneId(1),
        PlayerSkin::Warrior,
        inventory.clone(),
    ));

    let message = ServerMessages::SyncInventory(inventory.clone());
    let message = bincode::serialize(&message).unwrap();
    server.send_message(*left_client_id, ServerChannel::ServerMessages, message);

    let right_transform = Transform::from_xyz(1500., 50., Layers::Player.as_f32());
    commands.entity(*right_player_entity).insert((
        right_transform,
        PlayerInput::default(),
        Velocity::default(),
        GameSceneId(1),
        PlayerSkin::Monster,
        inventory.clone(),
    ));

    let message = ServerMessages::SyncInventory(inventory);
    let message = bincode::serialize(&message).unwrap();
    server.send_message(*right_client_id, ServerChannel::ServerMessages, message);

    let players = vec![
        SpawnPlayer {
            id: *left_client_id,
            entity: *left_player_entity,
            translation: left_transform.translation.into(),
            skin: PlayerSkin::Warrior,
        },
        SpawnPlayer {
            id: *right_client_id,
            entity: *right_player_entity,
            translation: right_transform.translation.into(),
            skin: PlayerSkin::Monster,
        },
    ];

    let message = ServerMessages::LoadGameScene {
        game_scene_type: GameSceneType::Fight,
        players,
        units: Vec::new(),
        projectiles: Vec::new(),
        flag: None,
    };
    let message = bincode::serialize(&message).unwrap();
    server.broadcast_message(ServerChannel::ServerMessages, message);
}

fn duel_map(
    lobby: &Res<ServerLobby>,
    game_world: &mut ResMut<GameWorld>,
    commands: &mut Commands,
    server: &mut ResMut<RenetServer>,
) {
    for (client_id, player_entity) in lobby.players.iter() {
        let (game_scene_id, skin, color, left_destination, right_destination) =
            if game_world.game_scenes.is_empty() {
                (
                    GameSceneId(1),
                    PlayerSkin::Warrior,
                    BLUE,
                    GameSceneDestination {
                        scene: GameSceneId(2),
                        position: Vec3::new(-1300., 50., Layers::Player.as_f32()),
                    },
                    GameSceneDestination {
                        scene: GameSceneId(2),
                        position: Vec3::new(1300., 50., Layers::Player.as_f32()),
                    },
                )
            } else {
                (
                    GameSceneId(2),
                    PlayerSkin::Monster,
                    RED,
                    GameSceneDestination {
                        scene: GameSceneId(1),
                        position: Vec3::new(-1300., 50., Layers::Player.as_f32()),
                    },
                    GameSceneDestination {
                        scene: GameSceneId(1),
                        position: Vec3::new(1300., 50., Layers::Player.as_f32()),
                    },
                )
            };
        println!("world: {:?}, skin: {:?}", game_scene_id, skin);

        // Create Game Scene
        let base = BaseScene::new();
        let server_components = (Owner(*client_id), game_scene_id);
        commands.spawn((base.main_building, server_components));
        commands.spawn((base.archer_building, server_components));
        commands.spawn((base.warrior_building, server_components));
        commands.spawn((base.pikeman_building, server_components));
        commands.spawn((base.left_wall, server_components));
        commands.spawn((base.right_wall, server_components));

        commands.spawn((base.left_spawn_point, server_components, left_destination));
        commands.spawn((base.right_spawn_point, server_components, right_destination));

        commands.spawn((base.left_gold_farm, server_components));
        commands.spawn((base.right_gold_farm, server_components));

        let game_scene_type = GameSceneType::Base(Color::from(color));
        let game_scene = GameScene {
            id: game_scene_id,
            game_scene_type,
            left_game_scenes: Vec::new(),
            right_game_scenes: Vec::new(),
        };
        game_world.game_scenes.insert(game_scene_id, game_scene);

        // Create Player entity
        let transform = Transform::from_xyz(0., 50., Layers::Player.as_f32());
        let inventory = Inventory { gold: 100 };
        commands.entity(*player_entity).insert((
            transform,
            PlayerInput::default(),
            Velocity::default(),
            game_scene_id,
            skin,
            inventory.clone(),
        ));

        let message = ServerMessages::LoadGameScene {
            game_scene_type,
            players: vec![SpawnPlayer {
                id: *client_id,
                entity: *player_entity,
                translation: transform.translation.into(),
                skin,
            }],
            units: Vec::new(),
            projectiles: Vec::new(),
            flag: None,
        };
        let message = bincode::serialize(&message).unwrap();
        server.send_message(*client_id, ServerChannel::ServerMessages, message);

        let message = ServerMessages::SyncInventory(inventory);
        let message = bincode::serialize(&message).unwrap();
        server.send_message(*client_id, ServerChannel::ServerMessages, message);
    }

    // setup duel map
    let mut iter = game_world.game_scenes.iter_mut();
    if let Some((first_game_scene_id, first_game_scene)) = iter.next() {
        if let Some((second_game_scene_id, second_game_scene)) = iter.next() {
            first_game_scene
                .left_game_scenes
                .push(*second_game_scene_id);
            first_game_scene
                .right_game_scenes
                .push(*second_game_scene_id);

            second_game_scene
                .left_game_scenes
                .push(*first_game_scene_id);
            second_game_scene
                .right_game_scenes
                .push(*first_game_scene_id);
        }
    }
}