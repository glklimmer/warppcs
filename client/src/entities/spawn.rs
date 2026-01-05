use bevy::{prelude::*, sprite::Anchor};

use animations::{
    animals::horse::{HorseAnimation, HorseSpriteSheet},
    king::{KingAnimation, KingSpriteSheet},
    objects::{
        chest::{ChestAnimation, ChestSpriteSheet},
        flag::{FlagAnimation, FlagSpriteSheet},
        portal::{PortalAnimation, PortalSpriteSheet},
        projectiles::{ProjectileSpriteSheet, Projectiles},
    },
    units::UnitSpriteSheets,
    world::{
        TreeAnimation,
        road::{RoadAnimation, RoadSpriteSheet},
        trees::pine::PineTreeSpriteSheet,
    },
};
use bevy_replicon::prelude::ClientTriggerExt;
use shared::{
    ClientReady, ControlledPlayer, Player, SetLocalPlayer,
    map::buildings::{Building, RecruitBuilding},
    player_port::Portal,
    server::{
        buildings::{recruiting::Flag, siege_camp::SiegeCamp},
        entities::{Unit, UnitAnimation, health::Health},
        physics::projectile::ProjectileType,
        players::{chest::Chest, mount::Mount},
    },
};
use sprite_variant_loader::loader::{SpriteVariants, SpriteVariantsAssetsExt};
use travel::{Road, SceneEnd};

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_player_sprite)
            .add_observer(init_recruit_building_sprite)
            .add_observer(init_camp_sprite)
            .add_observer(init_unit_sprite)
            .add_observer(init_flag_sprite)
            .add_observer(init_scene_end_sprite)
            .add_observer(init_portal_sprite)
            .add_observer(init_road_sprite)
            .add_observer(init_horse_sprite)
            .add_observer(init_projectile_sprite)
            .add_observer(init_chest_sprite);
    }
}

fn init_player_sprite(
    trigger: On<Add, Player>,
    mut players: Query<(&mut Sprite, &Player)>,
    king_sprite_sheet: Res<KingSpriteSheet>,
    variants: Res<Assets<SpriteVariants>>,
    mut commands: Commands,
) -> Result {
    let (mut sprite, player) = players.get_mut(trigger.entity)?;

    let handle = &king_sprite_sheet.sprite_sheet.texture;
    let sprite_variants = variants.get_variant(handle)?;
    let animation = king_sprite_sheet
        .sprite_sheet
        .animations
        .get(KingAnimation::Idle);

    sprite.image = sprite_variants.variants.get(player.color).clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: king_sprite_sheet.sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((animation.clone(), KingAnimation::default()));
    Ok(())
}

fn init_recruit_building_sprite(
    trigger: On<Add, RecruitBuilding>,
    mut slots: Query<&mut Sprite>,
    asset_server: Res<AssetServer>,
) -> Result {
    let mut sprite = slots.get_mut(trigger.entity)?;
    sprite.image = asset_server.load::<Image>(Building::marker_texture());
    Ok(())
}

fn init_camp_sprite(
    trigger: On<Add, SiegeCamp>,
    mut camp: Query<&mut Sprite>,
    asset_server: Res<AssetServer>,
) -> Result {
    let mut sprite = camp.get_mut(trigger.entity)?;
    sprite.image = asset_server.load::<Image>("sprites/buildings/siege_camp.png");
    Ok(())
}

fn init_unit_sprite(
    trigger: On<Add, Unit>,
    mut units: Query<(&mut Sprite, &Unit, Option<&Health>)>,
    sprite_sheets: Res<UnitSpriteSheets>,
    variants: Res<Assets<SpriteVariants>>,
    mut commands: Commands,
) -> Result {
    let (mut sprite, unit, maybe_health) = units.get_mut(trigger.entity)?;

    let sprite_sheet = &sprite_sheets.sprite_sheets.get(unit.unit_type);
    let handle = &sprite_sheet.texture;
    let sprite_variants = variants.get_variant(handle)?;
    let animation = match maybe_health {
        Some(_) => UnitAnimation::Idle,
        None => UnitAnimation::Death,
    };
    let sprite_sheet_animation = sprite_sheet.animations.get(animation);

    sprite.image = sprite_variants.variants.get(unit.color).clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: sprite_sheet_animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((sprite_sheet_animation.clone(), animation));
    Ok(())
}

fn init_flag_sprite(
    trigger: On<Add, Flag>,
    mut flag: Query<(&mut Sprite, &Flag)>,
    flag_sprite_sheet: Res<FlagSpriteSheet>,
    variants: Res<Assets<SpriteVariants>>,
    mut commands: Commands,
) -> Result {
    let (mut sprite, flag) = flag.get_mut(trigger.entity)?;

    let sprite_sheet = &flag_sprite_sheet.sprite_sheet;
    let handle = &sprite_sheet.texture;
    let sprite_variants = variants.get_variant(handle)?;
    let animation = sprite_sheet.animations.get(FlagAnimation::default());

    sprite.image = sprite_variants.variants.get(flag.color).clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((animation.clone(), FlagAnimation::default()));
    Ok(())
}

fn init_scene_end_sprite(
    trigger: On<Add, SceneEnd>,
    mut scene_end: Query<&mut Sprite>,
    tree_sprite_sheet: Res<PineTreeSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let mut sprite = scene_end.get_mut(trigger.entity)?;

    let bright_sprite_sheet = &tree_sprite_sheet.bright_sprite_sheet;

    let animation = bright_sprite_sheet.animations.get(TreeAnimation::default());
    let texture_atlas = Some(TextureAtlas {
        layout: bright_sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let bright_texture = &bright_sprite_sheet.texture;
    let dim_texture = &tree_sprite_sheet.dim_sprite_sheet.texture;
    let dark_texture = &tree_sprite_sheet.dark_sprite_sheet.texture;

    sprite.image = bright_texture.clone();
    sprite.texture_atlas = texture_atlas.clone();

    let mut entity_commands = commands.entity(trigger.entity);
    entity_commands.insert((animation.clone(), TreeAnimation::default()));

    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(-39., 0., 8.),
        Sprite {
            image: bright_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(-22., 1., 5.),
        Sprite {
            image: dim_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(-14., 0., 3.),
        Sprite {
            image: dim_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(-8., 0., 7.),
        Sprite {
            image: dark_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(8., 2., 6.),
        Sprite {
            image: dim_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(17., 1., 4.),
        Sprite {
            image: dark_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(25., 2., 1.),
        Sprite {
            image: bright_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    Ok(())
}

fn init_portal_sprite(
    trigger: On<Add, Portal>,
    mut portal: Query<&mut Sprite>,
    portal_sprite_sheet: Res<PortalSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let mut sprite = portal.get_mut(trigger.entity)?;

    let sprite_sheet = &portal_sprite_sheet.sprite_sheet;
    let animation = sprite_sheet.animations.get(PortalAnimation::default());

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((animation.clone(), PortalAnimation::default()));
    Ok(())
}

fn init_road_sprite(
    trigger: On<Add, Road>,
    mut road: Query<&mut Sprite>,
    road_sprite_sheet: Res<RoadSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let mut sprite = road.get_mut(trigger.entity)?;

    let sprite_sheet = &road_sprite_sheet.sprite_sheet;
    let animation = sprite_sheet.animations.get(RoadAnimation::default());

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((animation.clone(), RoadAnimation::default()));
    Ok(())
}

fn init_horse_sprite(
    trigger: On<Add, Mount>,
    mut portal: Query<&mut Sprite>,
    horse_sprite_sheet: Res<HorseSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let mut sprite = portal.get_mut(trigger.entity)?;

    let sprite_sheet = &horse_sprite_sheet.sprite_sheet;
    let animation = sprite_sheet.animations.get(HorseAnimation::default());

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((animation.clone(), HorseAnimation::default()));
    Ok(())
}

fn init_projectile_sprite(
    trigger: On<Add, ProjectileType>,
    mut projectile: Query<(&mut Sprite, &ProjectileType)>,
    projectiles: Res<ProjectileSpriteSheet>,
) -> Result {
    let (mut sprite, projectile_type) = projectile.get_mut(trigger.entity)?;

    let texture = match projectile_type {
        ProjectileType::Arrow => projectiles.sprite_sheet.texture_atlas(Projectiles::Arrow),
    };
    sprite.texture_atlas = Some(texture);
    sprite.image = projectiles.sprite_sheet.texture.clone();
    Ok(())
}

fn init_chest_sprite(
    trigger: On<Add, Chest>,
    mut chests: Query<&mut Sprite>,
    sprite_sheets: Res<ChestSpriteSheet>,
) -> Result {
    let mut sprite = chests.get_mut(trigger.entity)?;

    let sprite_sheet = &sprite_sheets.sprite_sheet;
    let animation = sprite_sheet.animations.get(ChestAnimation::Open);

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });
    Ok(())
}
