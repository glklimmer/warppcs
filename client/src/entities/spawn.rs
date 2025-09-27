use bevy::{prelude::*, sprite::Anchor};

use bevy_parallax::CameraFollow;
use shared::{
    ChestAnimation, FlagAnimation, Player, SetLocalPlayer,
    map::buildings::{Building, RecruitBuilding},
    server::{
        buildings::{recruiting::Flag, siege_camp::SiegeCamp},
        entities::{Unit, UnitAnimation},
        game_scenes::travel::{Portal, Road, SceneEnd},
        physics::projectile::ProjectileType,
        players::{chest::Chest, mount::Mount},
    },
};

use crate::{
    animations::{
        animals::horse::{HorseAnimation, HorseSpriteSheet},
        king::{KingAnimation, KingSpriteSheet},
        objects::{
            chest::ChestSpriteSheet,
            flag::FlagSpriteSheet,
            portal::{PortalAnimation, PortalSpriteSheet},
            projectiles::{ProjectileSpriteSheet, Projectiles},
        },
        sprite_variant_loader::SpriteVariants,
        units::UnitSpriteSheets,
        world::{
            TreeAnimation,
            road::{RoadAnimation, RoadSpriteSheet},
            trees::pine::PineTreeSpriteSheet,
        },
    },
    networking::ControlledPlayer,
};

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
            .add_observer(init_chest_sprite)
            .add_observer(init_local_player);
    }
}

fn init_local_player(
    trigger: Trigger<SetLocalPlayer>,
    mut commands: Commands,
    camera: Query<Entity, With<Camera>>,
) {
    let player = trigger.entity();
    let mut player_commands = commands.entity(player);
    player_commands.insert((ControlledPlayer, SpatialListener::new(50.0)));
    commands
        .entity(camera.single().unwrap())
        .insert(CameraFollow::fixed(player).with_offset(Vec2 { x: 0., y: 50. }));
}

fn init_player_sprite(
    trigger: Trigger<OnAdd, Player>,
    mut players: Query<(&mut Sprite, &Player)>,
    mut commands: Commands,
    king_sprite_sheet: Res<KingSpriteSheet>,
    variants: Res<Assets<SpriteVariants>>,
) {
    let Ok((mut sprite, player)) = players.get_mut(trigger.target()) else {
        return;
    };

    let handle = &king_sprite_sheet.sprite_sheet.texture;
    let sprite_variants = variants.get(handle).unwrap();
    let animation = king_sprite_sheet
        .sprite_sheet
        .animations
        .get(KingAnimation::Idle);

    sprite.image = sprite_variants.variants.get(player.color).clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: king_sprite_sheet.sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.target());
    commands.insert((animation.clone(), KingAnimation::default()));
}

fn init_recruit_building_sprite(
    trigger: Trigger<OnAdd, RecruitBuilding>,
    mut slots: Query<&mut Sprite>,
    asset_server: Res<AssetServer>,
) {
    let Ok(mut sprite) = slots.get_mut(trigger.target()) else {
        return;
    };

    sprite.image = asset_server.load::<Image>(Building::marker_texture());
}

fn init_camp_sprite(
    trigger: Trigger<OnAdd, SiegeCamp>,
    mut camp: Query<&mut Sprite>,
    asset_server: Res<AssetServer>,
) {
    let Ok(mut sprite) = camp.get_mut(trigger.target()) else {
        return;
    };
    info!("setting camp sprite");

    sprite.image = asset_server.load::<Image>("sprites/buildings/siege_camp.png");
}

fn init_unit_sprite(
    trigger: Trigger<OnAdd, Unit>,
    mut units: Query<(&mut Sprite, &Unit)>,
    sprite_sheets: Res<UnitSpriteSheets>,
    mut commands: Commands,
    variants: Res<Assets<SpriteVariants>>,
) {
    let Ok((mut sprite, unit)) = units.get_mut(trigger.target()) else {
        return;
    };

    let sprite_sheet = &sprite_sheets.sprite_sheets.get(unit.unit_type);
    let handle = &sprite_sheet.texture;
    let sprite_variants = variants.get(handle).unwrap();
    let animation = sprite_sheet.animations.get(UnitAnimation::Idle);

    sprite.image = sprite_variants.variants.get(unit.color).clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.target());
    commands.insert((animation.clone(), UnitAnimation::default()));
}

fn init_flag_sprite(
    trigger: Trigger<OnAdd, Flag>,
    mut commands: Commands,
    mut flag: Query<(&mut Sprite, &Flag)>,
    flag_sprite_sheet: Res<FlagSpriteSheet>,
    variants: Res<Assets<SpriteVariants>>,
) {
    let Ok((mut sprite, flag)) = flag.get_mut(trigger.target()) else {
        return;
    };

    let sprite_sheet = &flag_sprite_sheet.sprite_sheet;
    let handle = &sprite_sheet.texture;
    let sprite_variants = variants.get(handle).unwrap();
    let animation = sprite_sheet.animations.get(FlagAnimation::Wave);

    sprite.image = sprite_variants.variants.get(flag.color).clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.target());
    commands.insert((animation.clone(), FlagAnimation::default()));
}

fn init_scene_end_sprite(
    trigger: Trigger<OnAdd, SceneEnd>,
    mut commands: Commands,
    mut scene_end: Query<&mut Sprite>,
    tree_sprite_sheet: Res<PineTreeSpriteSheet>,
) {
    let Ok(mut sprite) = scene_end.get_mut(trigger.target()) else {
        return;
    };

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

    let mut entity_commands = commands.entity(trigger.target());
    entity_commands.insert((animation.clone(), TreeAnimation::default()));

    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(-39., 0., 8.),
        Sprite {
            image: bright_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            anchor: Anchor::BottomCenter,
            ..default()
        },
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(-22., 1., 5.),
        Sprite {
            image: dim_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            anchor: Anchor::BottomCenter,
            ..default()
        },
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(-14., 0., 3.),
        Sprite {
            image: dim_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            anchor: Anchor::BottomCenter,
            ..default()
        },
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(-8., 0., 7.),
        Sprite {
            image: dark_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            anchor: Anchor::BottomCenter,
            ..default()
        },
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(8., 2., 6.),
        Sprite {
            image: dim_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            anchor: Anchor::BottomCenter,
            ..default()
        },
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(17., 1., 4.),
        Sprite {
            image: dark_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            anchor: Anchor::BottomCenter,
            ..default()
        },
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(25., 2., 1.),
        Sprite {
            image: bright_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            anchor: Anchor::BottomCenter,
            ..default()
        },
    ));
}

fn init_portal_sprite(
    trigger: Trigger<OnAdd, Portal>,
    mut commands: Commands,
    mut portal: Query<&mut Sprite>,
    portal_sprite_sheet: Res<PortalSpriteSheet>,
) {
    let Ok(mut sprite) = portal.get_mut(trigger.target()) else {
        return;
    };

    let sprite_sheet = &portal_sprite_sheet.sprite_sheet;
    let animation = sprite_sheet.animations.get(PortalAnimation::default());

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.target());
    commands.insert((animation.clone(), PortalAnimation::default()));
}

fn init_road_sprite(
    trigger: Trigger<OnAdd, Road>,
    mut commands: Commands,
    mut road: Query<&mut Sprite>,
    road_sprite_sheet: Res<RoadSpriteSheet>,
) {
    let Ok(mut sprite) = road.get_mut(trigger.target()) else {
        return;
    };

    let sprite_sheet = &road_sprite_sheet.sprite_sheet;
    let animation = sprite_sheet.animations.get(RoadAnimation::default());

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.target());
    commands.insert((animation.clone(), RoadAnimation::default()));
}

fn init_horse_sprite(
    trigger: Trigger<OnAdd, Mount>,
    mut commands: Commands,
    mut portal: Query<&mut Sprite>,
    horse_sprite_sheet: Res<HorseSpriteSheet>,
) {
    let Ok(mut sprite) = portal.get_mut(trigger.target()) else {
        return;
    };

    let sprite_sheet = &horse_sprite_sheet.sprite_sheet;
    let animation = sprite_sheet.animations.get(HorseAnimation::default());

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.target());
    commands.insert((animation.clone(), HorseAnimation::default()));
}

fn init_projectile_sprite(
    trigger: Trigger<OnAdd, ProjectileType>,
    mut projectile: Query<(&mut Sprite, &ProjectileType)>,
    projectiles: Res<ProjectileSpriteSheet>,
) {
    let Ok((mut sprite, projectile_type)) = projectile.get_mut(trigger.target()) else {
        return;
    };

    let texture = match projectile_type {
        ProjectileType::Arrow => projectiles.sprite_sheet.texture_atlas(Projectiles::Arrow),
    };
    sprite.texture_atlas = Some(texture);
    sprite.image = projectiles.sprite_sheet.texture.clone();
}

fn init_chest_sprite(
    trigger: Trigger<OnAdd, Chest>,
    mut chests: Query<&mut Sprite>,
    sprite_sheets: Res<ChestSpriteSheet>,
) {
    let Ok(mut sprite) = chests.get_mut(trigger.target()) else {
        return;
    };

    let sprite_sheet = &sprite_sheets.sprite_sheet;
    let animation = sprite_sheet.animations.get(ChestAnimation::Open);

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });
}
