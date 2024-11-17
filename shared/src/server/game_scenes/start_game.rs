use bevy::prelude::*;

use bevy::color::palettes::css::{BLUE, RED};
use bevy_renet::renet::RenetServer;

use crate::map::base::BaseScene;
use crate::map::{GameScene, GameSceneType, Layers};
use crate::networking::{Owner, PlayerInput, ServerChannel, ServerMessages, SpawnPlayer};
use crate::server::economy::Inventory;
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

fn start_game(
    mut commands: Commands,
    mut network_events: EventReader<NetworkEvent>,
    mut game_world: ResMut<GameWorld>,
    mut server: ResMut<RenetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    lobby: Res<ServerLobby>,
) {
    for event in network_events.read() {
        if let PlayerCommand::StartGame = &event.message {
            #[cfg(prod)]
            if !game_lobby.all_ready() {
                continue;
            }
            println!("Starting game...");
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
                commands.spawn((base.left_gold_farm, server_components));
                commands.spawn((base.right_gold_farm, server_components));
                commands.spawn((base.left_spawn_point, server_components, left_destination));
                commands.spawn((base.right_spawn_point, server_components, right_destination));

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
                let gold_amount = Inventory { gold: 100 };
                commands.entity(*player_entity).insert((
                    transform,
                    PlayerInput::default(),
                    Velocity::default(),
                    game_scene_id,
                    skin,
                    gold_amount.clone(),
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

                let message = ServerMessages::SyncInventory(gold_amount);
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

            next_state.set(GameState::GameSession);
        }
    }
}
