use bevy::prelude::*;

use crate::{
    animations::{
        king::{KingAnimation, KingSpriteSheet},
        objects::flag::{FlagAnimation, FlagSpriteSheet},
        units::UnitSpriteSheets,
    },
    entities::highlight::Highlightable,
    networking::{ClientPlayers, ControlledPlayer, CurrentClientId, NetworkMapping},
};
use bevy_parallax::CameraFollow;
use shared::{
    map::{
        buildings::{BuildStatus, Building},
        Layers,
    },
    networking::{DropFlag, PickFlag},
    projectile_collider,
    server::{
        buildings::recruiting::Flag,
        entities::{Unit, UnitAnimation},
    },
    BoxCollider, LocalClientId, PhysicalPlayer,
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
            .add_observer(init_unit_sprite)
            .add_observer(init_flag_sprite)
            .add_systems(Update, update_building_sprite);

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
    trigger: Trigger<OnAdd, PhysicalPlayer>,
    mut players: Query<&mut Sprite>,
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
    let mut commands = commands.entity(trigger.entity());
    commands.insert((animation.clone(), KingAnimation::default()));
}

fn init_building_sprite(
    trigger: Trigger<OnAdd, Building>,
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

fn init_unit_sprite(
    trigger: Trigger<OnAdd, Unit>,
    mut units: Query<(&mut Sprite, &Unit)>,
    sprite_sheets: Res<UnitSpriteSheets>,
    mut commands: Commands,
) {
    let Ok((mut sprite, unit)) = units.get_mut(trigger.entity()) else {
        return;
    };

    let sprite_sheet = &sprite_sheets.sprite_sheets.get(unit.unit_type);
    sprite.image = sprite_sheet.texture.clone();
    let animation = sprite_sheet.animations.get(UnitAnimation::Idle);
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });
    let mut commands = commands.entity(trigger.entity());
    commands.insert((animation.clone(), UnitAnimation::default()));
}

fn init_flag_sprite(
    trigger: Trigger<OnAdd, Flag>,
    mut commands: Commands,
    mut flag: Query<&mut Sprite>,
    flag_sprite_sheet: Res<FlagSpriteSheet>,
) {
    let Ok((mut sprite)) = flag.get_mut(trigger.entity()) else {
        return;
    };

    print!("Flag");

    let sprite_sheet = &flag_sprite_sheet.sprite_sheet;
    sprite.image = sprite_sheet.texture.clone();
    let animation = sprite_sheet.animations.get(FlagAnimation::Wave);
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });
    let mut commands = commands.entity(trigger.entity());
    commands.insert((
        animation.clone(),
        FlagAnimation::default(),
        Highlightable::default(),
    ));
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
