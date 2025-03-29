use bevy::prelude::*;

use crate::{
    animations::{
        animals::horse::{HorseAnimation, HorseSpriteSheet},
        king::{KingAnimation, KingSpriteSheet},
        objects::{
            flag::{FlagAnimation, FlagSpriteSheet},
            portal::{PortalAnimation, PortalSpriteSheet},
        },
        units::UnitSpriteSheets,
    },
    networking::ControlledPlayer,
};
use bevy_parallax::CameraFollow;
use shared::{
    map::buildings::{BuildStatus, Building},
    server::{
        buildings::recruiting::Flag,
        entities::{Unit, UnitAnimation},
        game_scenes::Portal,
        physics::projectile::ProjectileType,
        players::mount::Mount,
    },
    LocalClientId, PhysicalPlayer,
};

use super::highlight::Highlighted;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_player_sprite)
            .add_observer(init_local_player)
            .add_observer(init_building_sprite)
            .add_observer(init_unit_sprite)
            .add_observer(init_flag_sprite)
            .add_observer(init_portal_sprite)
            .add_observer(init_horse_sprite)
            .add_observer(init_projectile_sprite)
            .add_systems(Update, update_building_sprite);
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
    mut buildings: Query<
        (
            &mut Sprite,
            &Building,
            &BuildStatus,
            Option<&mut Highlighted>,
        ),
        Changed<BuildStatus>,
    >,
    asset_server: Res<AssetServer>,
) {
    for (mut sprite, building, status, maybe_highlight) in buildings.iter_mut() {
        sprite.image = asset_server.load(building.texture(*status));

        if let Some(mut highlight) = maybe_highlight {
            highlight.original_handle = asset_server.load(building.texture(*status));
        }
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
    let Ok(mut sprite) = flag.get_mut(trigger.entity()) else {
        return;
    };

    let sprite_sheet = &flag_sprite_sheet.sprite_sheet;
    sprite.image = sprite_sheet.texture.clone();
    let animation = sprite_sheet.animations.get(FlagAnimation::Wave);
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });
    let mut commands = commands.entity(trigger.entity());
    commands.insert((animation.clone(), FlagAnimation::default()));
}

fn init_portal_sprite(
    trigger: Trigger<OnAdd, Portal>,
    mut commands: Commands,
    mut portal: Query<&mut Sprite>,
    portal_sprite_sheet: Res<PortalSpriteSheet>,
) {
    let Ok(mut sprite) = portal.get_mut(trigger.entity()) else {
        return;
    };

    let sprite_sheet = &portal_sprite_sheet.sprite_sheet;
    sprite.image = sprite_sheet.texture.clone();
    let animation = sprite_sheet.animations.get(PortalAnimation::default());
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });
    let mut commands = commands.entity(trigger.entity());
    commands.insert((animation.clone(), PortalAnimation::default()));
}

fn init_horse_sprite(
    trigger: Trigger<OnAdd, Mount>,
    mut commands: Commands,
    mut portal: Query<&mut Sprite>,
    horse_sprite_sheet: Res<HorseSpriteSheet>,
) {
    let Ok(mut sprite) = portal.get_mut(trigger.entity()) else {
        return;
    };

    let sprite_sheet = &horse_sprite_sheet.sprite_sheet;
    sprite.image = sprite_sheet.texture.clone();
    let animation = sprite_sheet.animations.get(HorseAnimation::default());
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });
    let mut commands = commands.entity(trigger.entity());
    commands.insert((animation.clone(), HorseAnimation::default()));
}

fn init_projectile_sprite(
    trigger: Trigger<OnAdd, ProjectileType>,
    mut projectile: Query<(&mut Sprite, &ProjectileType)>,
    asset_server: Res<AssetServer>,
) {
    let Ok((mut sprite, projectile_type)) = projectile.get_mut(trigger.entity()) else {
        return;
    };

    let texture = match projectile_type {
        ProjectileType::Arrow => asset_server.load("sprites/arrow.png"),
    };
    sprite.image = texture
}
