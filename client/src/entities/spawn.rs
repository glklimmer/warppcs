use bevy::prelude::*;

use crate::{
    animations::king::{KingAnimation, KingSpriteSheet},
    networking::{ClientPlayers, ControlledPlayer, CurrentClientId, NetworkMapping},
};
use bevy_parallax::CameraFollow;
use shared::{
    map::{
        buildings::{BuildStatus, Building},
        Layers,
    },
    networking::{DropFlag, PickFlag},
    projectile_collider, BoxCollider, LocalClientId, PhysicalPlayer,
};

use super::highlight::Highlighted;

pub struct SpawnPlugin;

#[derive(Component)]
#[require(BoxCollider(projectile_collider))]
pub struct Projectile;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_player_sprite)
            .add_observer(init_local_player)
            .add_observer(init_building_sprite)
            .add_systems(Update, update_building_sprite);

        //app.init_resource::<FlagSpriteSheet>()

        // app.add_event::<SpawnPlayer>();
        // app.add_event::<SpawnUnit>();
        // app.add_event::<SpawnMount>();
        // app.add_event::<SpawnProjectile>();
        // app.add_event::<SpawnFlag>();
        // app.add_event::<DropFlag>();
        // app.add_event::<PickFlag>();

        // app.add_systems(
        //     FixedPostUpdate,
        //     (
        //         spawn.run_if(on_event::<NetworkEvent>),
        //         (
        //             (
        //                 spawn_player.run_if(on_event::<SpawnPlayer>),
        //                 spawn_flag.run_if(on_event::<SpawnFlag>),
        //             )
        //                 .chain(),
        //             spawn_unit.run_if(on_event::<SpawnUnit>),
        //             spawn_mount.run_if(on_event::<SpawnMount>),
        //             spawn_projectile.run_if(on_event::<SpawnProjectile>),
        //         ),
        //     )
        //         .chain()
        //         .in_set(Connected),
        // );
        //
        // app.add_systems(FixedUpdate, drop_flag.run_if(on_event::<DropFlag>));
        // app.add_systems(FixedUpdate, pick_flag.run_if(on_event::<PickFlag>));
    }
}

fn init_local_player(
    trigger: Trigger<OnAdd, PhysicalPlayer>,
    mut commands: Commands,
    players: Query<&PhysicalPlayer>,
    client_id: Res<LocalClientId>,
    camera: Query<Entity, With<Camera>>,
) {
    let entity = trigger.entity();
    let player = players.get(entity).unwrap();

    if **player == **client_id {
        info!("init controlled player for {:?}", **player);
        let mut player_commands = commands.entity(entity);
        player_commands.insert(ControlledPlayer);
        commands
            .entity(camera.single())
            .insert(CameraFollow::fixed(entity));
    }
}

fn init_player_sprite(
    trigger: Trigger<OnAdd, Sprite>,
    mut players: Query<&mut Sprite, With<PhysicalPlayer>>,
    mut commands: Commands,
    king_sprite_sheet: Res<KingSpriteSheet>,
) {
    let Ok(mut sprite) = players.get_mut(trigger.entity()) else {
        return;
    };
    let sprite_sheet = &king_sprite_sheet.sprite_sheet;
    sprite.image = sprite_sheet.texture.clone();
    let animation = sprite_sheet.animations.get(KingAnimation::Idle);
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });
    let mut player_commands = commands.entity(trigger.entity());
    player_commands.insert((animation.clone(), KingAnimation::default()));
}

fn init_building_sprite(
    trigger: Trigger<OnAdd, Sprite>,
    mut buildings: Query<(&mut Sprite, &Building, &BuildStatus)>,
    asset_server: Res<AssetServer>,
) {
    let Ok((mut sprite, building, status)) = buildings.get_mut(trigger.entity()) else {
        return;
    };

    sprite.image = asset_server.load::<Image>(building.texture(*status));
}

fn update_building_sprite(
    mut buildings: Query<(&mut Sprite, &Building, &BuildStatus), Changed<BuildStatus>>,
    asset_server: Res<AssetServer>,
) {
    for (mut sprite, building, status) in buildings.iter_mut() {
        sprite.image = asset_server.load(building.texture(*status));
    }
}

// #[allow(clippy::too_many_arguments)]
// fn spawn(
//     mut network_events: EventReader<NetworkEvent>,
//     mut spawn_player: EventWriter<SpawnPlayer>,
//     mut spawn_unit: EventWriter<SpawnUnit>,
//     mut spawn_mount: EventWriter<SpawnMount>,
//     mut spawn_projectile: EventWriter<SpawnProjectile>,
//     mut spawn_flag: EventWriter<SpawnFlag>,
//     mut pick_flag: EventWriter<PickFlag>,
//     mut drop_flag: EventWriter<DropFlag>,
// ) {
//     for event in network_events.read() {
//         match &event.message {
//             ServerMessages::SpawnPlayer(spawn) => {
//                 spawn_player.send(spawn.clone());
//             }
//             ServerMessages::SpawnUnit(spawn) => {
//                 spawn_unit.send(spawn.clone());
//             }
//             ServerMessages::SpawnMount(spawn) => {
//                 spawn_mount.send(spawn.clone());
//             }
//             ServerMessages::SpawnProjectile(spawn) => {
//                 spawn_projectile.send(spawn.clone());
//             }
//             ServerMessages::SpawnFlag(spawn) => {
//                 spawn_flag.send(spawn.clone());
//             }
//             ServerMessages::PickFlag(pick) => {
//                 pick_flag.send(pick.clone());
//             }
//             ServerMessages::DropFlag(drop) => {
//                 drop_flag.send(drop.clone());
//             }
//             ServerMessages::SpawnGroup { player, units } => {
//                 spawn_player.send(player.clone());
//
//                 for unit in units {
//                     spawn_unit.send(unit.clone());
//                 }
//             }
//             _ => (),
//         }
//     }
// }
//
// fn spawn_player(
//     mut commands: Commands,
//     mut spawn_player: EventReader<SpawnPlayer>,
//     mut lobby: ResMut<ClientPlayers>,
//     mut network_mapping: ResMut<NetworkMapping>,
//     client_id: Res<CurrentClientId>,
//     king_sprite_sheet: Res<KingSpriteSheet>,
//     camera: Query<Entity, With<Camera>>,
// ) {
//     let client_id = client_id.0;
//     for spawn in spawn_player.read() {
//         let SpawnPlayer {
//             id,
//             translation,
//             entity: server_player_entity,
//             mounted,
//         } = spawn;
//
//         let mut client_player_entity = commands.spawn((
//             SpriteAnimationBundle::new(
//                 translation,
//                 &king_sprite_sheet.sprite_sheet,
//                 match mounted {
//                     Some(mounted) => match mounted.mount_type {
//                         MountType::Horse => KingAnimation::HorseIdle,
//                     },
//                     None => KingAnimation::Idle,
//                 },
//                 3.,
//             ),
//             PartOfScene,
//         ));
//
//         if let Some(mounted) = mounted {
//             client_player_entity.insert(Mounted {
//                 mount_type: mounted.mount_type,
//             });
//         }
//
//         let player_entity = client_player_entity.id();
//
//         if client_id.eq(id) {
//             client_player_entity.insert(ControlledPlayer);
//             commands
//                 .entity(camera.single())
//                 .insert(CameraFollow::fixed(player_entity));
//         }
//
//         let player_info = PlayerEntityMapping {
//             server_entity: *server_player_entity,
//             client_entity: player_entity,
//         };
//
//         lobby.players.insert(*id, player_info);
//         network_mapping
//             .0
//             .insert(*server_player_entity, player_entity);
//     }
// }
//
// fn spawn_unit(
//     mut commands: Commands,
//     mut spawn_unit: EventReader<SpawnUnit>,
//     mut network_mapping: ResMut<NetworkMapping>,
//     sprite_sheets: Res<UnitSpriteSheets>,
// ) {
//     for spawn in spawn_unit.read() {
//         let SpawnUnit {
//             entity: server_unit_entity,
//             owner,
//             translation,
//             unit_type,
//         } = spawn;
//
//         let sprite_sheet = sprite_sheets.sprite_sheets.get(*unit_type);
//
//         let client_unit_entity = commands
//             .spawn((
//                 Unit,
//                 SpriteAnimationBundle::new(translation, sprite_sheet, UnitAnimation::Idle, 3.),
//                 *unit_type,
//                 *owner,
//             ))
//             .id();
//
//         network_mapping
//             .0
//             .insert(*server_unit_entity, client_unit_entity);
//     }
// }
//
// fn spawn_mount(
//     mut commands: Commands,
//     mut spawn_unit: EventReader<SpawnMount>,
//     mut network_mapping: ResMut<NetworkMapping>,
//     sprite_sheet: Res<HorseSpriteSheet>,
// ) {
//     for spawn in spawn_unit.read() {
//         let SpawnMount {
//             entity: server_unit_entity,
//             mount_type,
//             translation,
//         } = spawn;
//
//         let client_unit_entity = commands
//             .spawn((
//                 Unit,
//                 SpriteAnimationBundle::new(
//                     translation,
//                     &sprite_sheet.sprite_sheet,
//                     HorseAnimation::Idle,
//                     3.,
//                 ),
//                 *mount_type,
//             ))
//             .id();
//
//         network_mapping
//             .0
//             .insert(*server_unit_entity, client_unit_entity);
//     }
// }
//
// fn spawn_projectile(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut network_mapping: ResMut<NetworkMapping>,
//     mut spawn_projectile: EventReader<SpawnProjectile>,
// ) {
//     for spawn in spawn_projectile.read() {
//         let SpawnProjectile {
//             entity: server_entity,
//             projectile_type,
//             translation,
//             direction,
//         } = spawn;
//         let texture = match projectile_type {
//             ProjectileType::Arrow => asset_server.load("sprites/arrow.png"),
//         };
//
//         let direction: Vec2 = (*direction).into();
//         let position: Vec3 = (*translation).into();
//         let position = position.truncate();
//
//         let angle = (direction - position).angle_to(position);
//
//         let client_entity = commands
//             .spawn((
//                 Projectile,
//                 Sprite {
//                     image: texture,
//                     ..default()
//                 },
//                 Transform {
//                     translation: (*translation).into(),
//                     scale: Vec3::splat(2.0),
//                     rotation: Quat::from_rotation_z(angle),
//                 },
//             ))
//             .id();
//
//         network_mapping.0.insert(*server_entity, client_entity);
//     }
// }
//
// fn spawn_flag(
//     mut commands: Commands,
//     mut network_mapping: ResMut<NetworkMapping>,
//     mut spawn_flag: EventReader<SpawnFlag>,
//     flag_sprite_sheet: Res<FlagSpriteSheet>,
//     client_id: Res<CurrentClientId>,
//     lobby: Res<ClientPlayers>,
// ) {
//     let client_id = client_id.0;
//     for spawn in spawn_flag.read() {
//         let SpawnFlag {
//             flag: server_flag_entity,
//         } = spawn;
//
//         let client_flag_entity = commands
//             .spawn((
//                 Flag,
//                 SpriteAnimationBundle::new(
//                     &[0., 0., Layers::Flag.as_f32()],
//                     &flag_sprite_sheet.sprite_sheet,
//                     FlagAnimation::Wave,
//                     0.2,
//                 ),
//             ))
//             .id();
//
//         let player_entity = lobby.players.get(&client_id).unwrap().client_entity;
//         if let Some(mut player) = commands.get_entity(player_entity) {
//             player.add_child(client_flag_entity);
//         }
//
//         network_mapping
//             .0
//             .insert(*server_flag_entity, client_flag_entity);
//     }
// }

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
