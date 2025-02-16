use bevy::prelude::*;

use bevy_renet::renet::RenetServer;
use std::env;

use crate::{
    entities::{Faction, MountType, Owner, UnitType},
    map::{
        scenes::{base::define_base_scene, camp::define_camp_scene, GameSceneId},
        Layers,
    },
    networking::{ServerChannel, ServerMessages, SpawnPlayer},
    player::{Inventory, PlayerCommand, PlayerInput},
    server::{
        ai::{
            attack::{unit_health, unit_swing_timer},
            UnitBehaviour,
        },
        buildings::recruiting::FlagAssignment,
        entities::{health::Health, Unit},
        game_scenes::GameSceneDestination,
        lobby::GameLobby,
        networking::{GameWorld, NetworkEvent, ServerLobby},
        physics::movement::{Speed, Velocity},
        players::{
            interaction::{Interactable, InteractionType},
            mount::Mount,
        },
    },
};

pub struct StartGamePlugin;

impl Plugin for StartGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (start_game).run_if(on_event::<NetworkEvent>));
    }
}

#[allow(clippy::too_many_arguments)]
fn start_game(
    mut commands: Commands,
    mut network_events: EventReader<NetworkEvent>,
    mut game_world: ResMut<GameWorld>,
    mut server: ResMut<RenetServer>,
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
                //     fight_map(&lobby, &mut commands, &mut server);
            } else {
                circle_map(&lobby, &mut game_world, &mut commands, &mut server);
            }
        }
    }
}

// fn fight_map(lobby: &Res<ServerLobby>, commands: &mut Commands, server: &mut ResMut<RenetServer>) {
//     let mut players: Vec<(&ClientId, &Entity)> = lobby.players.iter().collect();
//     let (left_client_id, left_player_entity) = players.pop().unwrap();
//     let (right_client_id, right_player_entity) = players.pop().unwrap();
//
//     // Create Fight Scene
//     let base = FightScene::new();
//     let server_components = (
//         Owner {
//             faction: Faction::Player {
//                 client_id: *left_client_id,
//             },
//         },
//         GameSceneId(1),
//     );
//     commands.spawn((
//         base.left_main_building,
//         server_components,
//         SceneSlotIndicator::Fight(FightSceneIndicator::LeftMainBuilding),
//     ));
//     commands.spawn((
//         base.left_archer_building,
//         server_components,
//         SceneSlotIndicator::Fight(FightSceneIndicator::LeftArcherBuilding),
//     ));
//     commands.spawn((
//         base.left_warrior_building,
//         server_components,
//         SceneSlotIndicator::Fight(FightSceneIndicator::LeftWarriorBuilding),
//     ));
//     commands.spawn((
//         base.left_pikeman_building,
//         server_components,
//         SceneSlotIndicator::Fight(FightSceneIndicator::LeftPikemanBuilding),
//     ));
//     commands.spawn((
//         base.left_left_wall,
//         server_components,
//         building_health(&base.left_left_wall.building),
//         SceneSlotIndicator::Fight(FightSceneIndicator::LeftLeftWall),
//     ));
//     commands.spawn((
//         base.left_right_wall,
//         server_components,
//         building_health(&base.left_right_wall.building),
//         SceneSlotIndicator::Fight(FightSceneIndicator::LeftRightWall),
//     ));
//     commands.spawn((
//         base.left_gold_farm,
//         server_components,
//         SceneSlotIndicator::Fight(FightSceneIndicator::LeftGoldFarm),
//     ));
//
//     let server_components = (
//         Owner {
//             faction: Faction::Player {
//                 client_id: *right_client_id,
//             },
//         },
//         GameSceneId(1),
//     );
//     commands.spawn((
//         base.right_main_building,
//         server_components,
//         SceneSlotIndicator::Fight(FightSceneIndicator::RightMainBuilding),
//     ));
//     commands.spawn((
//         base.right_archer_building,
//         server_components,
//         SceneSlotIndicator::Fight(FightSceneIndicator::RightArcherBuilding),
//     ));
//     commands.spawn((
//         base.right_warrior_building,
//         server_components,
//         SceneSlotIndicator::Fight(FightSceneIndicator::RightWarriorBuilding),
//     ));
//     commands.spawn((
//         base.right_pikeman_building,
//         server_components,
//         SceneSlotIndicator::Fight(FightSceneIndicator::RightPikemanBuilding),
//     ));
//     commands.spawn((
//         base.right_left_wall,
//         server_components,
//         building_health(&base.right_left_wall.building),
//         SceneSlotIndicator::Fight(FightSceneIndicator::RightLeftWall),
//     ));
//     commands.spawn((
//         base.right_right_wall,
//         server_components,
//         building_health(&base.right_right_wall.building),
//         SceneSlotIndicator::Fight(FightSceneIndicator::RightRightWall),
//     ));
//     commands.spawn((
//         base.right_gold_farm,
//         server_components,
//         SceneSlotIndicator::Fight(FightSceneIndicator::RightGoldFarm),
//     ));
//
//     // Create Player entities
//     let left_transform = Transform::from_xyz(-200., 50., Layers::Player.as_f32());
//     let inventory = Inventory { gold: 1000 };
//     commands.entity(*left_player_entity).insert((
//         left_transform,
//         GameSceneId(1),
//         inventory.clone(),
//         Owner {
//             faction: Faction::Player {
//                 client_id: *left_client_id,
//             },
//         },
//     ));
//
//     let message = ServerMessages::SyncInventory(inventory.clone());
//     let message = bincode::serialize(&message).unwrap();
//     server.send_message(*left_client_id, ServerChannel::ServerMessages, message);
//
//     let right_transform = Transform::from_xyz(200., 50., Layers::Player.as_f32());
//     commands.entity(*right_player_entity).insert((
//         right_transform,
//         GameSceneId(1),
//         inventory.clone(),
//         Owner {
//             faction: Faction::Player {
//                 client_id: *right_client_id,
//             },
//         },
//     ));
//
//     let message = ServerMessages::SyncInventory(inventory);
//     let message = bincode::serialize(&message).unwrap();
//     server.send_message(*right_client_id, ServerChannel::ServerMessages, message);
//
//     let players = vec![
//         SpawnPlayer {
//             id: *left_client_id,
//             entity: *left_player_entity,
//             translation: left_transform.translation.into(),
//             mounted: None,
//         },
//         SpawnPlayer {
//             id: *right_client_id,
//             entity: *right_player_entity,
//             translation: right_transform.translation.into(),
//             mounted: None,
//         },
//     ];
//
//     let message = ServerMessages::LoadGameScene {
//         game_scene_type: GameSceneType::Fight,
//         players,
//         units: Vec::new(),
//         mounts: Vec::new(),
//         projectiles: Vec::new(),
//         buildings: Vec::new(),
//         flag: None,
//     };
//     let message = bincode::serialize(&message).unwrap();
//     server.broadcast_message(ServerChannel::ServerMessages, message);
// }

fn circle_map(
    lobby: &Res<ServerLobby>,
    game_world: &mut ResMut<GameWorld>,
    commands: &mut Commands,
    server: &mut ResMut<RenetServer>,
) {
    for (index, (client_id, player_entity)) in lobby.players.iter().enumerate() {
        let game_scene_index = index * 2;

        // Create Base Scene
        {
            let game_scene_id = GameSceneId(game_scene_index);
            let left_destination = GameSceneDestination {
                scene: GameSceneId(game_scene_index - 1 % 10),
                position: Vec3::new(-800., 50., Layers::Player.as_f32()),
            };
            let right_destination = GameSceneDestination {
                scene: GameSceneId(game_scene_index + 1 % 10),
                position: Vec3::new(800., 50., Layers::Player.as_f32()),
            };

            let owner = Owner {
                faction: Faction::Player {
                    client_id: *client_id,
                },
            };
            let base = define_base_scene(game_scene_id);

            for slot in base.slots {
                let entity = commands
                    .spawn((game_scene_id, slot.transform, slot.collider))
                    .id();
                (slot.spawn_fn)(&mut commands.entity(entity), owner);
            }

            let left_portal = commands
                .spawn((
                    game_scene_id,
                    base.left_portal.transform,
                    base.left_portal.collider,
                    left_destination,
                ))
                .id();
            (base.left_portal.spawn_fn)(&mut commands.entity(left_portal), owner);
            let right_portal = commands
                .spawn((
                    game_scene_id,
                    base.right_portal.transform,
                    base.right_portal.collider,
                    right_destination,
                ))
                .id();
            (base.right_portal.spawn_fn)(&mut commands.entity(right_portal), owner);

            game_world.game_scenes.insert(game_scene_id, base);

            // Create Player entity
            let transform = Transform::from_xyz(0., 50., Layers::Player.as_f32());
            let inventory = Inventory::default();
            commands.entity(*player_entity).insert((
                transform,
                PlayerInput::default(),
                Velocity::default(),
                Speed::default(),
                game_scene_id,
                inventory.clone(),
                Owner {
                    faction: Faction::Player {
                        client_id: *client_id,
                    },
                },
            ));

            let message = ServerMessages::LoadGameScene {
                game_scene_type: base.game_scene_type,
                players: vec![SpawnPlayer {
                    id: *client_id,
                    entity: *player_entity,
                    translation: transform.translation.into(),
                    mounted: None,
                }],
                units: Vec::new(),
                projectiles: Vec::new(),
                mounts: Vec::new(),
                buildings: Vec::new(),
                flag: None,
            };
            let message = bincode::serialize(&message).unwrap();
            server.send_message(*client_id, ServerChannel::ServerMessages, message);

            let message = ServerMessages::SyncInventory(inventory);
            let message = bincode::serialize(&message).unwrap();
            server.send_message(*client_id, ServerChannel::ServerMessages, message);
        }

        // Spawn Camp to the right
        {
            let camp_index = game_scene_index + 1;
            let game_scene_id = GameSceneId(camp_index);
            let left_destination = GameSceneDestination {
                scene: GameSceneId(camp_index - 1 % 10),
                position: Vec3::new(-1800., 50., Layers::Player.as_f32()),
            };
            let right_destination = GameSceneDestination {
                scene: GameSceneId(camp_index + 1 % 10),
                position: Vec3::new(1800., 50., Layers::Player.as_f32()),
            };

            let owner = Owner {
                faction: Faction::Bandits,
            };
            let camp = define_camp_scene(game_scene_id);

            for slot in camp.slots {
                let entity = commands
                    .spawn((game_scene_id, slot.transform, slot.collider))
                    .id();
                (slot.spawn_fn)(&mut commands.entity(entity), owner);
            }

            let left_portal = commands
                .spawn((
                    game_scene_id,
                    camp.left_portal.transform,
                    camp.left_portal.collider,
                    left_destination,
                ))
                .id();
            (camp.left_portal.spawn_fn)(&mut commands.entity(left_portal), owner);
            let right_portal = commands
                .spawn((
                    game_scene_id,
                    camp.right_portal.transform,
                    camp.right_portal.collider,
                    right_destination,
                ))
                .id();
            (camp.right_portal.spawn_fn)(&mut commands.entity(right_portal), owner);

            // Spawn Mount
            commands.spawn((
                Transform::from_xyz(1400., 45., Layers::Unit.as_f32()),
                Mount {
                    mount_type: MountType::Horse,
                },
                Interactable {
                    kind: InteractionType::Mount,
                    restricted_to: None,
                },
                game_scene_id,
            ));

            // Spawn Bandits
            let flag_entity = commands
                .spawn((Transform::from_translation(Vec3::ZERO),))
                .id();

            let unit_type = UnitType::Bandit;
            let transform = Transform::from_xyz(0., 50., Layers::Unit.as_f32());

            for unit_number in 1..=4 {
                let offset = Vec2::new(40. * (unit_number - 3) as f32 + 20., 0.);
                commands.spawn((
                    transform,
                    Unit {
                        unit_type,
                        swing_timer: unit_swing_timer(&unit_type),
                    },
                    Health {
                        hitpoints: unit_health(&unit_type),
                    },
                    Owner {
                        faction: Faction::Bandits,
                    },
                    FlagAssignment(flag_entity, offset),
                    UnitBehaviour::FollowFlag(flag_entity, offset),
                    game_scene_id,
                ));
            }

            game_world.game_scenes.insert(game_scene_id, camp);
        }
    }

    // // setup duel map
    // let first_base_id = GameSceneId(1);
    // let second_base_id = GameSceneId(2);
    // let first_camp_id = GameSceneId(3);
    // let second_camp_id = GameSceneId(4);
    //
    // let first_base = game_world.game_scenes.get_mut(&first_base_id).unwrap();
    // first_base.left_game_scenes.push(first_camp_id);
    // first_base.right_game_scenes.push(second_camp_id);
    //
    // let second_base = game_world.game_scenes.get_mut(&second_base_id).unwrap();
    // second_base.left_game_scenes.push(second_camp_id);
    // second_base.right_game_scenes.push(first_camp_id);
    //
    // let first_camp = game_world.game_scenes.get_mut(&first_camp_id).unwrap();
    // first_camp.left_game_scenes.push(second_base_id);
    // first_camp.right_game_scenes.push(first_base_id);
    //
    // let second_camp = game_world.game_scenes.get_mut(&second_camp_id).unwrap();
    // second_camp.left_game_scenes.push(first_base_id);
    // second_camp.right_game_scenes.push(second_base_id);
}
