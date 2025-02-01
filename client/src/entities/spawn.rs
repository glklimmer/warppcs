use bevy::prelude::*;
use bevy_parallax::CameraFollow;

use crate::{
    animations::{
        king::{KingAnimation, KingSpriteSheet},
        objects::flag::{Flag, FlagAnimation, FlagSpriteSheet},
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
        DropFlag, PickFlag, ProjectileType, ServerMessages, SpawnFlag, SpawnPlayer,
        SpawnProjectile, SpawnUnit,
    },
    projectile_collider, BoxCollider,
};

use super::{highlight::Highlighted, PartOfScene};

pub struct SpawnPlugin;

#[derive(Component)]
#[require(PartOfScene, BoxCollider(projectile_collider))]
pub struct Projectile;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlagSpriteSheet>();

        app.add_event::<SpawnPlayer>();
        app.add_event::<SpawnUnit>();
        app.add_event::<SpawnProjectile>();
        app.add_event::<SpawnFlag>();
        app.add_event::<DropFlag>();
        app.add_event::<PickFlag>();

        app.add_systems(
            FixedUpdate,
            (
                spawn.run_if(on_event::<NetworkEvent>),
                (
                    (
                        spawn_player.run_if(on_event::<SpawnPlayer>),
                        spawn_flag.run_if(on_event::<SpawnFlag>),
                    )
                        .chain(),
                    spawn_unit.run_if(on_event::<SpawnUnit>),
                    spawn_projectile.run_if(on_event::<SpawnProjectile>),
                ),
            )
                .chain()
                .in_set(Connected),
        );

        app.add_systems(FixedUpdate, drop_flag.run_if(on_event::<DropFlag>));
        app.add_systems(FixedUpdate, pick_flag.run_if(on_event::<PickFlag>));
    }
}

fn spawn(
    mut network_events: EventReader<NetworkEvent>,
    mut spawn_player: EventWriter<SpawnPlayer>,
    mut spawn_unit: EventWriter<SpawnUnit>,
    mut spawn_projectile: EventWriter<SpawnProjectile>,
    mut spawn_flag: EventWriter<SpawnFlag>,
    mut pick_flag: EventWriter<PickFlag>,
    mut drop_flag: EventWriter<DropFlag>,
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
            ServerMessages::PickFlag(pick) => {
                pick_flag.send(pick.clone());
            }
            ServerMessages::DropFlag(drop) => {
                drop_flag.send(drop.clone());
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
    camera: Query<Entity, With<Camera>>,
) {
    let client_id = client_id.0;
    for spawn in spawn_player.read() {
        let SpawnPlayer {
            id,
            translation,
            entity: server_player_entity,
        } = spawn;

        let mut client_player_entity = commands.spawn((
            SpriteAnimationBundle::new(
                translation,
                &king_sprite_sheet.sprite_sheet,
                KingAnimation::Idle,
                3.,
            ),
            PartOfScene,
        ));
        let player_entity = client_player_entity.id();

        if client_id.eq(id) {
            client_player_entity.insert(ControlledPlayer);
            commands
                .entity(camera.single())
                .insert(CameraFollow::fixed(player_entity));
        }

        let player_info = PlayerEntityMapping {
            server_entity: *server_player_entity,
            client_entity: player_entity,
        };

        lobby.players.insert(*id, player_info);
        network_mapping
            .0
            .insert(*server_player_entity, player_entity);
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
                Projectile,
                Sprite {
                    image: texture,
                    ..default()
                },
                Transform {
                    translation: (*translation).into(),
                    scale: Vec3::splat(2.0),
                    rotation: Quat::from_rotation_z(angle),
                },
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
            flag: server_flag_entity,
        } = spawn;

        let client_flag_entity = commands
            .spawn((
                Flag,
                SpriteAnimationBundle::new(
                    &[0., 0., Layers::Flag.as_f32()],
                    &flag_sprite_sheet.sprite_sheet,
                    FlagAnimation::Wave,
                    0.2,
                ),
            ))
            .id();

        let player_entity = lobby.players.get(&client_id).unwrap().client_entity;
        commands.entity(player_entity).add_child(client_flag_entity);

        network_mapping
            .0
            .insert(*server_flag_entity, client_flag_entity);
    }
}

fn pick_flag(
    mut commands: Commands,
    network_mapping: ResMut<NetworkMapping>,
    mut pick_flag: EventReader<PickFlag>,
    client_id: Res<CurrentClientId>,
    lobby: Res<ClientPlayers>,
) {
    for flag in pick_flag.read() {
        let PickFlag {
            flag: server_flag_entity,
        } = flag;
        let client_id = client_id.0;

        let player_entity = lobby.players.get(&client_id).unwrap().client_entity;
        let client_flag_entity = network_mapping.0.get(server_flag_entity).unwrap();

        commands
            .entity(*client_flag_entity)
            .insert(Transform {
                translation: Vec3::new(0., 0., Layers::Flag.as_f32()),
                scale: Vec3::splat(0.2),
                ..default()
            })
            .remove::<Highlighted>()
            .set_parent(player_entity);
    }
}

fn drop_flag(
    mut commands: Commands,
    network_mapping: Res<NetworkMapping>,
    mut drop_flag: EventReader<DropFlag>,
) {
    for drop in drop_flag.read() {
        let DropFlag {
            flag: server_flag_entity,
            translation,
        } = drop;
        let client_flag_entity = network_mapping.0.get(server_flag_entity).unwrap();

        commands
            .entity(*client_flag_entity)
            .remove_parent()
            .insert((Transform::from_translation(*translation),));
    }
}
