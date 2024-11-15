use bevy::prelude::*;

use crate::{
    animations::{
        animation::UnitAnimation,
        king::{PaladinBundle, PaladinSpriteSheet, WarriorBundle, WarriorSpriteSheet},
        FlagBundle, FlagSpriteSheet,
    },
    networking::{
        ClientPlayers, Connected, ControlledPlayer, CurrentClientId, NetworkEvent, NetworkMapping,
        PlayerEntityMapping,
    },
};
use shared::{
    networking::{
        PlayerSkin, ProjectileType, ServerMessages, SpawnFlag, SpawnPlayer, SpawnProjectile,
        SpawnUnit, UnitType,
    },
    BoxCollider,
};

use super::PartOfScene;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlagSpriteSheet>();

        app.add_event::<SpawnPlayer>();
        app.add_event::<SpawnUnit>();
        app.add_event::<SpawnProjectile>();
        app.add_event::<SpawnFlag>();

        app.add_systems(
            FixedUpdate,
            (
                spawn,
                (
                    (spawn_player, spawn_flag).chain(),
                    spawn_unit,
                    spawn_projectile,
                )
                    .chain(),
            )
                .run_if(on_event::<NetworkEvent>())
                .in_set(Connected),
        );
    }
}

fn spawn(
    mut network_events: EventReader<NetworkEvent>,
    mut spawn_player: EventWriter<SpawnPlayer>,
    mut spawn_unit: EventWriter<SpawnUnit>,
    mut spawn_projectile: EventWriter<SpawnProjectile>,
    mut spawn_flag: EventWriter<SpawnFlag>,
) {
    for event in network_events.read() {
        match &event.message {
            ServerMessages::SpawnPlayer(spawn) => {
                spawn_player.send(spawn.clone());
            }
            ServerMessages::SpawnUnit(spawn) => {
                spawn_unit.send(spawn.clone());
            }
            ServerMessages::SpawnProjectile(spawn) => {
                spawn_projectile.send(spawn.clone());
            }
            ServerMessages::SpawnFlag(spawn) => {
                spawn_flag.send(spawn.clone());
            }
            ServerMessages::SpawnGroup { player, units } => {
                spawn_player.send(player.clone());

                for unit in units {
                    spawn_unit.send(unit.clone());
                }
            }
            _ => (),
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    mut spawn_player: EventReader<SpawnPlayer>,
    client_id: Res<CurrentClientId>,
    warrior_sprite_sheet: Res<WarriorSpriteSheet>,
    paladin_sprite_sheet: Res<PaladinSpriteSheet>,
    mut lobby: ResMut<ClientPlayers>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let client_id = client_id.0;
    for spawn in spawn_player.read() {
        let SpawnPlayer {
            id,
            translation,
            entity: server_player_entity,
            skin,
        } = spawn;

        let mut client_player_entity = match skin {
            PlayerSkin::Warrior => commands.spawn((
                PaladinBundle::new(&paladin_sprite_sheet, *translation, UnitAnimation::Idle),
                PartOfScene,
            )),
            PlayerSkin::Monster => commands.spawn((
                WarriorBundle::new(&warrior_sprite_sheet, *translation, UnitAnimation::Idle),
                PartOfScene,
            )),
        };

        if client_id.eq(id) {
            client_player_entity.insert((ControlledPlayer, BoxCollider(Vec2::new(50., 90.))));
        }

        let player_info = PlayerEntityMapping {
            server_entity: *server_player_entity,
            client_entity: client_player_entity.id(),
        };

        lobby.players.insert(*id, player_info);
        network_mapping
            .0
            .insert(*server_player_entity, client_player_entity.id());
    }
}

fn spawn_unit(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_unit: EventReader<SpawnUnit>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    for spawn in spawn_unit.read() {
        let SpawnUnit {
            entity: server_unit_entity,
            owner,
            translation,
            unit_type,
        } = spawn;

        let texture = match unit_type {
            UnitType::Shieldwarrior => asset_server.load("aseprite/shield_warrior.png"),
            UnitType::Pikeman => asset_server.load("aseprite/pike_man.png"),
            UnitType::Archer => asset_server.load("aseprite/archer.png"),
        };

        let client_unit_entity = commands
            .spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: (*translation).into(),
                        scale: Vec3::splat(3.0),
                        ..default()
                    },
                    texture,
                    ..default()
                },
                *owner,
                PartOfScene,
            ))
            .id();

        network_mapping
            .0
            .insert(*server_unit_entity, client_unit_entity);
    }
}

fn spawn_projectile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut spawn_projectile: EventReader<SpawnProjectile>,
) {
    for spawn in spawn_projectile.read() {
        let SpawnProjectile {
            entity: server_entity,
            projectile_type,
            translation,
            direction,
        } = spawn;
        let texture = match projectile_type {
            ProjectileType::Arrow => asset_server.load("aseprite/arrow.png"),
        };

        let direction: Vec2 = (*direction).into();
        let position: Vec3 = (*translation).into();
        let position = position.truncate();

        let angle = (direction - position).angle_between(position);

        let client_entity = commands
            .spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: (*translation).into(),
                        scale: Vec3::splat(2.0),
                        rotation: Quat::from_rotation_z(angle),
                    },
                    texture,
                    ..default()
                },
                PartOfScene,
            ))
            .id();

        network_mapping.0.insert(*server_entity, client_entity);
    }
}

fn spawn_flag(
    mut commands: Commands,
    mut network_mapping: ResMut<NetworkMapping>,
    mut spawn_flag: EventReader<SpawnFlag>,
    flag_sprite_sheet: Res<FlagSpriteSheet>,
    client_id: Res<CurrentClientId>,
    lobby: Res<ClientPlayers>,
) {
    let client_id = client_id.0;
    for spawn in spawn_flag.read() {
        let SpawnFlag {
            entity: server_flag_entity,
        } = spawn;

        let client_flag_entity = commands
            .spawn((FlagBundle::new(&flag_sprite_sheet), PartOfScene))
            .id();

        let player_entity = lobby.players.get(&client_id).unwrap().client_entity;
        commands.entity(player_entity).add_child(client_flag_entity);

        network_mapping
            .0
            .insert(*server_flag_entity, client_flag_entity);
    }
}
