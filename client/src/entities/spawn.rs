use bevy::prelude::*;

use crate::{
    animations::{
        king::{KingAnimation, KingSpriteSheet},
        units::{Unit, UnitAnimation, UnitSpriteSheets},
        SpriteAnimationBundle,
    },
    networking::{
        ClientPlayers, Connected, ControlledPlayer, CurrentClientId, NetworkEvent, NetworkMapping,
        PlayerEntityMapping,
    },
};
use shared::{
    map::Layers,
    networking::{
        ProjectileType, ServerMessages, SpawnFlag, SpawnPlayer, SpawnProjectile, SpawnUnit,
    },
    PROJECTILE_COLLIDER,
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
                .run_if(on_event::<NetworkEvent>)
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
    mut lobby: ResMut<ClientPlayers>,
    mut network_mapping: ResMut<NetworkMapping>,
    client_id: Res<CurrentClientId>,
    king_sprite_sheet: Res<KingSpriteSheet>,
) {
    let client_id = client_id.0;
    for spawn in spawn_player.read() {
        let SpawnPlayer {
            id,
            translation,
            entity: server_player_entity,
            skin,
        } = spawn;

        let mut client_player_entity = commands.spawn((SpriteAnimationBundle::new(
            translation,
            &king_sprite_sheet.sprite_sheet,
            KingAnimation::Idle,
            3.,
        ),));

        if client_id.eq(id) {
            client_player_entity.insert(ControlledPlayer);
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
    mut spawn_unit: EventReader<SpawnUnit>,
    mut network_mapping: ResMut<NetworkMapping>,
    sprite_sheets: Res<UnitSpriteSheets>,
) {
    for spawn in spawn_unit.read() {
        let SpawnUnit {
            entity: server_unit_entity,
            owner,
            translation,
            unit_type,
        } = spawn;

        let sprite_sheet = sprite_sheets.sprite_sheets.get(*unit_type);

        let client_unit_entity = commands
            .spawn((
                Unit,
                SpriteAnimationBundle::new(translation, sprite_sheet, UnitAnimation::Idle, 3.),
                *unit_type,
                *owner,
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
            ProjectileType::Arrow => asset_server.load("sprites/arrow.png"),
        };

        let direction: Vec2 = (*direction).into();
        let position: Vec3 = (*translation).into();
        let position = position.truncate();

        let angle = (direction - position).angle_to(position);

        let client_entity = commands
            .spawn((
                Sprite {
                    image: texture,
                    ..default()
                },
                Transform {
                    translation: (*translation).into(),
                    scale: Vec3::splat(2.0),
                    rotation: Quat::from_rotation_z(angle),
                },
                PROJECTILE_COLLIDER,
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
            .spawn((
                SpriteAnimationBundle::new(
                    &[0., 0., Layers::Flag.as_f32()],
                    &flag_sprite_sheet.sprite_sheet,
                    FlagAnimation::Wave,
                    0.2,
                ),
                FlagAnimation::Wave,
                PartOfScene,
            ))
            .id();

        let player_entity = lobby.players.get(&client_id).unwrap().client_entity;
        commands.entity(player_entity).add_child(client_flag_entity);

        network_mapping
            .0
            .insert(*server_flag_entity, client_flag_entity);
    }
}
