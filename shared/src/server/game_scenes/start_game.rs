use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::{
    map::{
        buildings::{BuildStatus, Building, MainBuildingLevels, RecruitBuilding, WallLevels},
        Layers,
    },
    networking::LobbyEvent,
    server::players::interaction::{Interactable, InteractionType},
    Faction, Owner, PhysicalPlayer,
};

pub struct StartGamePlugin;

impl Plugin for StartGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            start_game
                .after(ServerSet::Receive)
                .run_if(server_or_singleplayer),
        );
    }
}

fn start_game(
    mut lobby_events: EventReader<FromClient<LobbyEvent>>,
    mut players: Query<(Entity, &mut Transform), With<PhysicalPlayer>>,
    mut commands: Commands,
) {
    for FromClient {
        client_id: _,
        event,
    } in lobby_events.read()
    {
        if let LobbyEvent::StartGame = &event {
            for (i, (player, mut transform)) in players.iter_mut().enumerate() {
                info!("Creating base for player {}", i);
                let offset = Vec3::new(10000. * i as f32, 0., Layers::Player.as_f32());
                transform.translation = offset;

                player_base(
                    commands.reborrow(),
                    offset.with_z(Layers::Building.as_f32()),
                    player,
                );
            }
        }
    }
}

fn player_base(mut commands: Commands, building_offset: Vec3, player: Entity) {
    let owner = Owner(Faction::Player(player));

    commands.spawn((
        Building::MainBuilding {
            level: MainBuildingLevels::Tent,
        },
        Building::MainBuilding {
            level: MainBuildingLevels::Tent,
        }
        .collider(),
        BuildStatus::Built,
        Transform::from_translation(building_offset),
        owner,
    ));
    commands.spawn((
        Building::Archer,
        RecruitBuilding,
        Transform::from_translation(Vec3::ZERO.with_x(135.) + building_offset),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Building::Warrior,
        RecruitBuilding,
        Transform::from_translation(Vec3::ZERO.with_x(-135.) + building_offset),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Building::Pikeman,
        RecruitBuilding,
        Transform::from_translation(Vec3::ZERO.with_x(235.) + building_offset),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Building::Wall {
            level: WallLevels::Basic,
        },
        Transform::from_translation(Vec3::ZERO.with_x(390.) + building_offset),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Building::Wall {
            level: WallLevels::Basic,
        },
        Transform::from_translation(Vec3::ZERO.with_x(-345.) + building_offset),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Building::GoldFarm,
        Transform::from_translation(Vec3::ZERO.with_x(320.) + building_offset),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));
    commands.spawn((
        Building::GoldFarm,
        Transform::from_translation(Vec3::ZERO.with_x(-265.) + building_offset),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(owner),
        },
    ));
}

//#[allow(clippy::too_many_arguments)]
// fn start_game(
//     mut commands: Commands,
//     mut lobby_events: EventReader<FromClient<LobbyEvent>>,
//     mut game_world: ResMut<GameWorld>,
//     mut server: ResMut<RenetServer>,
//     lobby: Res<ServerLobby>,
//     game_lobby: Res<GameLobby>,
// ) {
//     println!("Lobby event received..");
//     for FromClient {
//         client_id: _,
//         event,
//     } in lobby_events.read()
//     {
//         info!("Lobby event received: {:?}", event);
//         if let LobbyEvent::StartGame = &event {
//             if !game_lobby.all_ready() {
//                 //continue;
//             }
//             println!("Starting game...");
//             let args: Vec<String> = env::args().collect();
//             if args.contains(&String::from("fight")) {
//                 fight_map(&lobby, &mut commands, &mut server);
//             } else {
//                 duel_map(&lobby, &mut game_world, &mut commands, &mut server);
//             }
//         }
//     }
// }

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
//         SceneBuildingIndicator::Fight(FightSceneIndicator::LeftMainBuilding),
//     ));
//     commands.spawn((
//         base.left_archer_building,
//         server_components,
//         SceneBuildingIndicator::Fight(FightSceneIndicator::LeftArcherBuilding),
//     ));
//     commands.spawn((
//         base.left_warrior_building,
//         server_components,
//         SceneBuildingIndicator::Fight(FightSceneIndicator::LeftWarriorBuilding),
//     ));
//     commands.spawn((
//         base.left_pikeman_building,
//         server_components,
//         SceneBuildingIndicator::Fight(FightSceneIndicator::LeftPikemanBuilding),
//     ));
//     commands.spawn((
//         base.left_left_wall,
//         server_components,
//         building_health(&base.left_left_wall.building),
//         SceneBuildingIndicator::Fight(FightSceneIndicator::LeftLeftWall),
//     ));
//     commands.spawn((
//         base.left_right_wall,
//         server_components,
//         building_health(&base.left_right_wall.building),
//         SceneBuildingIndicator::Fight(FightSceneIndicator::LeftRightWall),
//     ));
//     commands.spawn((
//         base.left_gold_farm,
//         server_components,
//         SceneBuildingIndicator::Fight(FightSceneIndicator::LeftGoldFarm),
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
//         SceneBuildingIndicator::Fight(FightSceneIndicator::RightMainBuilding),
//     ));
//     commands.spawn((
//         base.right_archer_building,
//         server_components,
//         SceneBuildingIndicator::Fight(FightSceneIndicator::RightArcherBuilding),
//     ));
//     commands.spawn((
//         base.right_warrior_building,
//         server_components,
//         SceneBuildingIndicator::Fight(FightSceneIndicator::RightWarriorBuilding),
//     ));
//     commands.spawn((
//         base.right_pikeman_building,
//         server_components,
//         SceneBuildingIndicator::Fight(FightSceneIndicator::RightPikemanBuilding),
//     ));
//     commands.spawn((
//         base.right_left_wall,
//         server_components,
//         building_health(&base.right_left_wall.building),
//         SceneBuildingIndicator::Fight(FightSceneIndicator::RightLeftWall),
//     ));
//     commands.spawn((
//         base.right_right_wall,
//         server_components,
//         building_health(&base.right_right_wall.building),
//         SceneBuildingIndicator::Fight(FightSceneIndicator::RightRightWall),
//     ));
//     commands.spawn((
//         base.right_gold_farm,
//         server_components,
//         SceneBuildingIndicator::Fight(FightSceneIndicator::RightGoldFarm),
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

// fn duel_map(
//     lobby: &Res<ServerLobby>,
//     game_world: &mut ResMut<GameWorld>,
//     commands: &mut Commands,
//     server: &mut ResMut<RenetServer>,
// ) {
//     for (client_id, player_entity) in lobby.players.iter() {
//         let (game_scene_id, left_destination, right_destination) =
//             if game_world.game_scenes.is_empty() {
//                 (
//                     GameSceneId(1),
//                     GameSceneDestination {
//                         scene: GameSceneId(3),
//                         position: Vec3::new(-800., 50., Layers::Player.as_f32()),
//                     },
//                     GameSceneDestination {
//                         scene: GameSceneId(4),
//                         position: Vec3::new(800., 50., Layers::Player.as_f32()),
//                     },
//                 )
//             } else {
//                 (
//                     GameSceneId(2),
//                     GameSceneDestination {
//                         scene: GameSceneId(4),
//                         position: Vec3::new(-800., 50., Layers::Player.as_f32()),
//                     },
//                     GameSceneDestination {
//                         scene: GameSceneId(3),
//                         position: Vec3::new(800., 50., Layers::Player.as_f32()),
//                     },
//                 )
//             };
//         println!("world: {:?} ", game_scene_id);
//
//         // Create Game Scene
//         let base = BaseScene::new();
//         let owner = Owner {
//             faction: Faction::Player {
//                 client_id: *client_id,
//             },
//         };
//         let server_components = (owner, game_scene_id);
//         commands.spawn((
//             base.main_building,
//             server_components,
//             SceneBuildingIndicator::Base(BaseSceneIndicator::MainBuilding),
//         ));
//         commands.spawn((
//             base.archer_building,
//             server_components,
//             RecruitBuilding,
//             Interactable {
//                 kind: InteractionType::Building,
//                 restricted_to: Some(owner),
//             },
//             SceneBuildingIndicator::Base(BaseSceneIndicator::ArcherBuilding),
//         ));
//         commands.spawn((
//             base.warrior_building,
//             server_components,
//             RecruitBuilding,
//             Interactable {
//                 kind: InteractionType::Building,
//                 restricted_to: Some(owner),
//             },
//             SceneBuildingIndicator::Base(BaseSceneIndicator::WarriorBuilding),
//         ));
//         commands.spawn((
//             base.pikeman_building,
//             server_components,
//             RecruitBuilding,
//             Interactable {
//                 kind: InteractionType::Building,
//                 restricted_to: Some(owner),
//             },
//             SceneBuildingIndicator::Base(BaseSceneIndicator::PikemanBuilding),
//         ));
//         commands.spawn((
//             base.left_wall,
//             server_components,
//             Interactable {
//                 kind: InteractionType::Building,
//                 restricted_to: Some(owner),
//             },
//             SceneBuildingIndicator::Base(BaseSceneIndicator::LeftWall),
//             building_health(&base.left_wall.building),
//         ));
//         commands.spawn((
//             base.right_wall,
//             server_components,
//             Interactable {
//                 kind: InteractionType::Building,
//                 restricted_to: Some(owner),
//             },
//             SceneBuildingIndicator::Base(BaseSceneIndicator::RightWall),
//             building_health(&base.right_wall.building),
//         ));
//
//         commands.spawn((
//             base.left_gold_farm,
//             server_components,
//             Interactable {
//                 kind: InteractionType::Building,
//                 restricted_to: Some(owner),
//             },
//             SceneBuildingIndicator::Base(BaseSceneIndicator::LeftGoldFarm),
//         ));
//         commands.spawn((
//             base.right_gold_farm,
//             server_components,
//             Interactable {
//                 kind: InteractionType::Building,
//                 restricted_to: Some(owner),
//             },
//             SceneBuildingIndicator::Base(BaseSceneIndicator::RightGoldFarm),
//         ));
//
//         commands.spawn((
//             base.left_spawn_point,
//             server_components,
//             left_destination,
//             Interactable {
//                 kind: InteractionType::Travel,
//                 restricted_to: None,
//             },
//             SceneBuildingIndicator::Base(BaseSceneIndicator::LeftSpawnPoint),
//         ));
//         commands.spawn((
//             base.right_spawn_point,
//             server_components,
//             right_destination,
//             Interactable {
//                 kind: InteractionType::Travel,
//                 restricted_to: None,
//             },
//             SceneBuildingIndicator::Base(BaseSceneIndicator::RightSpawnPoint),
//         ));
//
//         let game_scene_type = GameSceneType::Base;
//         let game_scene = GameScene {
//             id: game_scene_id,
//             game_scene_type,
//             left_game_scenes: Vec::new(),
//             right_game_scenes: Vec::new(),
//         };
//         game_world.game_scenes.insert(game_scene_id, game_scene);
//
//         // Create Player entity
//         let transform = Transform::from_xyz(0., 50., Layers::Player.as_f32());
//         let inventory = Inventory::default();
//         commands.entity(*player_entity).insert((
//             transform,
//             PlayerInput::default(),
//             Velocity::default(),
//             Speed::default(),
//             game_scene_id,
//             inventory.clone(),
//             Owner {
//                 faction: Faction::Player {
//                     client_id: *client_id,
//                 },
//             },
//         ));
//
//         let message = ServerMessages::LoadGameScene {
//             game_scene_type,
//             players: vec![SpawnPlayer {
//                 id: *client_id,
//                 entity: *player_entity,
//                 translation: transform.translation.into(),
//                 mounted: None,
//             }],
//             units: Vec::new(),
//             projectiles: Vec::new(),
//             mounts: Vec::new(),
//             buildings: Vec::new(),
//             flag: None,
//         };
//         let message = bincode::serialize(&message).unwrap();
//         server.send_message(*client_id, ServerChannel::ServerMessages, message);
//
//         let message = ServerMessages::SyncInventory(inventory);
//         let message = bincode::serialize(&message).unwrap();
//         server.send_message(*client_id, ServerChannel::ServerMessages, message);
//     }
//
//     for i in 3..=4 {
//         let base = CampScene::new();
//         let game_scene_id = GameSceneId(i);
//         let owner = Owner {
//             faction: Faction::Bandits,
//         };
//         let server_components = (owner, game_scene_id);
//         commands.spawn((
//             base.chest,
//             server_components,
//             Interactable {
//                 kind: InteractionType::Chest,
//                 restricted_to: Some(owner),
//             },
//             SceneBuildingIndicator::Camp(CampSceneIndicator::Chest),
//         ));
//         commands.spawn((
//             base.left_spawn_point,
//             server_components,
//             GameSceneDestination {
//                 scene: GameSceneId(if i == 3 { 1 } else { 2 }),
//                 position: Vec3::new(-1800., 50., Layers::Chest.as_f32()),
//             },
//             Interactable {
//                 kind: InteractionType::Travel,
//                 restricted_to: None,
//             },
//             SceneBuildingIndicator::Camp(CampSceneIndicator::LeftSpawn),
//         ));
//         commands.spawn((
//             base.right_spawn_point,
//             server_components,
//             GameSceneDestination {
//                 scene: GameSceneId(if i == 3 { 2 } else { 1 }),
//                 position: Vec3::new(1800., 50., Layers::Chest.as_f32()),
//             },
//             Interactable {
//                 kind: InteractionType::Travel,
//                 restricted_to: None,
//             },
//             SceneBuildingIndicator::Camp(CampSceneIndicator::RightSpawn),
//         ));
//         commands.spawn((
//             Transform::from_xyz(1400., 45., Layers::Unit.as_f32()),
//             Mount {
//                 mount_type: MountType::Horse,
//             },
//             Interactable {
//                 kind: InteractionType::Mount,
//                 restricted_to: None,
//             },
//             game_scene_id,
//         ));
//
//         let flag_entity = commands
//             .spawn((Transform::from_translation(Vec3::ZERO),))
//             .id();
//
//         let unit_type = UnitType::Bandit;
//         let transform = Transform::from_xyz(0., 50., Layers::Unit.as_f32());
//
//         for unit_number in 1..=4 {
//             let offset = Vec2::new(40. * (unit_number - 3) as f32 + 20., 0.);
//             commands.spawn((
//                 transform,
//                 Unit {
//                     unit_type,
//                     swing_timer: unit_swing_timer(&unit_type),
//                 },
//                 Health {
//                     hitpoints: unit_health(&unit_type),
//                 },
//                 Owner {
//                     faction: Faction::Bandits,
//                 },
//                 FlagAssignment(flag_entity, offset),
//                 UnitBehaviour::FollowFlag(flag_entity, offset),
//                 game_scene_id,
//             ));
//         }
//
//         let game_scene_type = GameSceneType::Camp;
//         let game_scene = GameScene {
//             id: game_scene_id,
//             game_scene_type,
//             left_game_scenes: Vec::new(),
//             right_game_scenes: Vec::new(),
//         };
//         game_world.game_scenes.insert(game_scene_id, game_scene);
//     }
//
//     // setup duel map
//     let first_base_id = GameSceneId(1);
//     let second_base_id = GameSceneId(2);
//     let first_camp_id = GameSceneId(3);
//     let second_camp_id = GameSceneId(4);
//
//     let first_base = game_world.game_scenes.get_mut(&first_base_id).unwrap();
//     first_base.left_game_scenes.push(first_camp_id);
//     first_base.right_game_scenes.push(second_camp_id);
//
//     let second_base = game_world.game_scenes.get_mut(&second_base_id).unwrap();
//     second_base.left_game_scenes.push(second_camp_id);
//     second_base.right_game_scenes.push(first_camp_id);
//
//     let first_camp = game_world.game_scenes.get_mut(&first_camp_id).unwrap();
//     first_camp.left_game_scenes.push(second_base_id);
//     first_camp.right_game_scenes.push(first_base_id);
//
//     let second_camp = game_world.game_scenes.get_mut(&second_camp_id).unwrap();
//     second_camp.left_game_scenes.push(first_base_id);
//     second_camp.right_game_scenes.push(second_base_id);
// }
