use bevy::prelude::*;

use bevy_parallax::CameraFollow;
use bevy_replicon::client::ClientSet;
use shared::{
    ChestAnimation, Player, SetLocalPlayer,
    map::buildings::{BuildStatus, Building, RecruitBuilding},
    server::{
        buildings::recruiting::Flag,
        entities::{Unit, UnitAnimation},
        game_scenes::travel::Portal,
        physics::projectile::ProjectileType,
        players::{chest::Chest, mount::Mount},
    },
};

use crate::{
    animations::{
        AnimationSound, AnimationSoundTrigger,
        animals::horse::{HorseAnimation, HorseSpriteSheet},
        king::{KingAnimation, KingSpriteSheet},
        objects::{
            chest::ChestSpriteSheet,
            flag::{FlagAnimation, FlagSpriteSheet},
            portal::{PortalAnimation, PortalSpriteSheet},
        },
        units::UnitSpriteSheets,
    },
    networking::ControlledPlayer,
    sound::CRAFTING_SOUND_PATH,
};

use super::highlight::Highlighted;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_player_sprite)
            .add_observer(init_recruit_building_sprite)
            .add_observer(init_building_sprite)
            .add_observer(init_unit_sprite)
            .add_observer(init_flag_sprite)
            .add_observer(init_portal_sprite)
            .add_observer(init_horse_sprite)
            .add_observer(init_projectile_sprite)
            .add_observer(init_chest_sprite)
            .add_systems(Update, update_building_sprite)
            .add_systems(PreUpdate, init_local_player.after(ClientSet::Receive));
    }
}

fn init_local_player(
    mut commands: Commands,
    mut events: EventReader<SetLocalPlayer>,
    camera: Query<Entity, With<Camera>>,
) {
    for event in events.read() {
        let player = **event;

        info!("init controlled player for {:?}", player);
        let mut player_commands = commands.entity(player);
        player_commands.insert((ControlledPlayer, SpatialListener::new(50.0)));
        commands
            .entity(camera.single())
            .insert(CameraFollow::fixed(player).with_offset(Vec2 { x: 0., y: 50. }));
    }
}

fn init_player_sprite(
    trigger: Trigger<OnAdd, Player>,
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

fn init_recruit_building_sprite(
    trigger: Trigger<OnAdd, RecruitBuilding>,
    mut slots: Query<(&mut Sprite, &BuildStatus, Option<&Building>)>,
    asset_server: Res<AssetServer>,
) {
    let Ok((mut sprite, status, maybe_building)) = slots.get_mut(trigger.entity()) else {
        return;
    };

    if let Some(building) = maybe_building {
        sprite.image = asset_server.load::<Image>(building.texture(*status));
    } else {
        sprite.image = asset_server.load::<Image>(Building::marker_texture());
    }
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
            Entity,
            &mut Sprite,
            &Building,
            &BuildStatus,
            Option<&mut Highlighted>,
        ),
        Or<(Changed<Building>, Changed<BuildStatus>)>,
    >,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (entity, mut sprite, building, status, maybe_highlight) in buildings.iter_mut() {
        sprite.image = asset_server.load(building.texture(*status));

        if let Some(mut highlight) = maybe_highlight {
            highlight.original_handle = asset_server.load(building.texture(*status));
        }
        if status.eq(&BuildStatus::Built) {
            commands.entity(entity).insert(AnimationSound {
                sound_handles: vec![
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/hammer_1.ogg"
                    )),
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/hammer_2.ogg"
                    )),
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/sawing_wood_1.ogg"
                    )),
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/sawing_wood_2.ogg"
                    )),
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/sawing_wood_3.ogg"
                    )),
                    asset_server.load(format!(
                        "{CRAFTING_SOUND_PATH}/hammering_&_sawing/hammering_&_chiseling_stone_1.ogg"
                    )),
                ],
                sound_trigger: AnimationSoundTrigger::OnEnter,
            });
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
        ProjectileType::Arrow => asset_server.load("sprites/objects/arrow.png"),
    };
    sprite.image = texture
}

fn init_chest_sprite(
    trigger: Trigger<OnAdd, Chest>,
    mut chests: Query<&mut Sprite>,
    sprite_sheets: Res<ChestSpriteSheet>,
) {
    let Ok(mut sprite) = chests.get_mut(trigger.entity()) else {
        return;
    };

    let sprite_sheet = &sprite_sheets.sprite_sheet;
    sprite.image = sprite_sheet.texture.clone();
    let animation = sprite_sheet.animations.get(ChestAnimation::Open);
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });
}
