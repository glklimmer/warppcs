use bevy::prelude::*;

use bevy_renet::renet::RenetServer;

use crate::{
    entities::{Faction, MountType, Owner, UnitType},
    map::{
        scenes::{base::define_base_scene, camp::define_camp_scene, GameSceneId},
        Layers,
    },
    networking::{ServerChannel, ServerMessages, Slot, SlotEntity, SlotType, SpawnPlayer},
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
            circle_map(&lobby, &mut game_world, &mut commands, &mut server);
        }
    }
}

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
            let base = define_base_scene();

            let mut slots: Vec<SlotEntity>;
            for prefab in base.slots {
                let slot = Slot {
                    slot_type: prefab.slot_type,
                };
                let mut entity =
                    commands.spawn((slot, game_scene_id, prefab.transform, prefab.collider));
                if let SlotType::Building { building, status } = prefab.slot_type {
                    entity.insert(status);
                    if let Some(building) = building {
                        entity.insert(building);
                    }
                }
                let entity = entity.id();
                (prefab.spawn_fn)(&mut commands.entity(entity), owner);
                slots.push(SlotEntity {
                    entity,
                    slot,
                    translation: prefab.transform.translation.into(),
                });
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
            slots.push(SlotEntity {
                entity: left_portal,
                slot: Slot {
                    slot_type: SlotType::Portal,
                },
                translation: base.left_portal.transform.translation.into(),
            });
            let right_portal = commands
                .spawn((
                    game_scene_id,
                    base.right_portal.transform,
                    base.right_portal.collider,
                    right_destination,
                ))
                .id();
            (base.right_portal.spawn_fn)(&mut commands.entity(right_portal), owner);
            slots.push(SlotEntity {
                entity: right_portal,
                slot: Slot {
                    slot_type: SlotType::Portal,
                },
                translation: base.right_portal.transform.translation.into(),
            });

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
                flag: None,
                slots,
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
            let camp = define_camp_scene();

            for slot in camp.slots {
                let slot_comp = Slot {
                    slot_type: SlotType::Chest,
                };
                let entity = commands
                    .spawn((slot_comp, game_scene_id, slot.transform, slot.collider))
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
