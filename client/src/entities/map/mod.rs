use bevy::prelude::*;

use bevy::color::palettes::css::YELLOW;
use shared::{
    map::{
        buildings::{BuildStatus, Building, BuildingBundle, MainBuildingLevels, WallLevels},
        scenes::{
            base::{BaseScene, BaseSceneIndicator},
            camp::{CampScene, CampSceneIndicator},
            fight::{FightScene, FightSceneIndicator},
            SceneBuildingIndicator,
        },
        GameSceneType,
    },
    networking::{
        BuildingUpdate, LoadBuilding, ServerMessages, SpawnFlag, SpawnPlayer, SpawnProjectile,
        SpawnUnit, UpdateType,
    },
    server::buildings::building_collider,
    GameState,
};

use crate::{
    animations::objects::chest::ChestSpriteSheet,
    networking::{Connected, NetworkEvent},
};

use super::PartOfScene;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (load_game_scene, update_building)
                .run_if(on_event::<NetworkEvent>)
                .in_set(Connected),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn load_game_scene(
    mut commands: Commands,
    mut network_events: EventReader<NetworkEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut spawn_player: EventWriter<SpawnPlayer>,
    mut spawn_unit: EventWriter<SpawnUnit>,
    mut spawn_projectile: EventWriter<SpawnProjectile>,
    mut spawn_flag: EventWriter<SpawnFlag>,
    entities: Query<Entity, With<PartOfScene>>,
    asset_server: Res<AssetServer>,
    chest_sprite_sheet: Res<ChestSpriteSheet>,
) {
    for event in network_events.read() {
        if let ServerMessages::LoadGameScene {
            game_scene_type: map_type,
            players,
            flag,
            units,
            projectiles,
            buildings,
        } = &event.message
        {
            println!("Loading map {:?}...", map_type);

            for entity in entities.iter() {
                commands.entity(entity).despawn();
            }

            match map_type {
                GameSceneType::Fight => {
                    let fight = FightScene::new();
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.left_main_building,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::LeftMainBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.left_archer_building,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::LeftArcherBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.left_warrior_building,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::LeftWarriorBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.left_pikeman_building,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::LeftPikemanBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.left_left_wall,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::LeftLeftWall),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.left_right_wall,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::LeftRightWall),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.left_gold_farm,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::LeftGoldFarm),
                    );

                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.right_main_building,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::RightMainBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.right_archer_building,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::RightArcherBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.right_warrior_building,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::RightWarriorBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.right_pikeman_building,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::RightPikemanBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.right_left_wall,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::RightLeftWall),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.right_right_wall,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::RightRightWall),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        fight.right_gold_farm,
                        SceneBuildingIndicator::Fight(FightSceneIndicator::RightGoldFarm),
                    );
                }
                GameSceneType::Base(color) => {
                    let base = BaseScene::new();

                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        base.main_building,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::MainBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        base.archer_building,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::ArcherBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        base.warrior_building,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::WarriorBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        base.pikeman_building,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::PikemanBuilding),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        base.left_wall,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::LeftWall),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        base.right_wall,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::RightWall),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        base.left_gold_farm,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::LeftGoldFarm),
                    );
                    spawn_building(
                        buildings,
                        &mut commands,
                        &asset_server,
                        base.right_gold_farm,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::RightGoldFarm),
                    );

                    commands.spawn((
                        base.left_spawn_point,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::LeftSpawnPoint),
                        (
                            Mesh2d(meshes.add(Rectangle::from_size(
                                base.left_spawn_point.collider.dimension,
                            ))),
                            MeshMaterial2d(materials.add(Color::from(YELLOW))),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        base.right_spawn_point,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::RightSpawnPoint),
                        (
                            Mesh2d(meshes.add(Rectangle::from_size(
                                base.left_spawn_point.collider.dimension,
                            ))),
                            MeshMaterial2d(materials.add(Color::from(YELLOW))),
                        ),
                        PartOfScene,
                    ));
                }
                GameSceneType::Camp => {
                    let camp = CampScene::new();
                    let sprite_sheet = &chest_sprite_sheet.sprite_sheet;
                    commands.spawn((
                        camp.chest,
                        SceneBuildingIndicator::Camp(CampSceneIndicator::Chest),
                        Sprite {
                            image: sprite_sheet.texture.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: sprite_sheet.layout.clone(),
                                index: 0,
                            }),
                            ..default()
                        },
                        PartOfScene,
                    ));

                    commands.spawn((
                        camp.left_spawn_point,
                        SceneBuildingIndicator::Camp(CampSceneIndicator::LeftSpawn),
                        (
                            Mesh2d(meshes.add(Rectangle::from_size(
                                camp.left_spawn_point.collider.dimension,
                            ))),
                            MeshMaterial2d(materials.add(Color::from(YELLOW))),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        camp.right_spawn_point,
                        SceneBuildingIndicator::Camp(CampSceneIndicator::RightSpawn),
                        (
                            Mesh2d(meshes.add(Rectangle::from_size(
                                camp.right_spawn_point.collider.dimension,
                            ))),
                            MeshMaterial2d(materials.add(Color::from(YELLOW))),
                        ),
                        PartOfScene,
                    ));
                }
            };
            players.iter().for_each(|spawn| {
                spawn_player.send(spawn.clone());
            });
            if let Some(spawn) = flag {
                spawn_flag.send(spawn.clone());
            }
            units.iter().for_each(|spawn| {
                spawn_unit.send(spawn.clone());
            });
            projectiles.iter().for_each(|spawn| {
                spawn_projectile.send(spawn.clone());
            });

            game_state.set(GameState::GameSession);
        }
    }
}

fn spawn_building(
    buildings: &[LoadBuilding],
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    building_bundle: BuildingBundle,
    indicator: SceneBuildingIndicator,
) {
    let load = buildings
        .iter()
        .find(|update| update.indicator.eq(&indicator));
    let load = match load {
        Some(load) => load,
        None => &LoadBuilding {
            indicator,
            status: building_bundle.build_status,
            upgrade: building_bundle.building,
        },
    };
    commands.spawn((
        building_bundle,
        indicator,
        Sprite {
            image: asset_server.load::<Image>(building_texture(&load.upgrade, load.status)),
            flip_x: building_flipped(&indicator),
            ..default()
        },
        PartOfScene,
    ));
}

fn update_building(
    mut network_events: EventReader<NetworkEvent>,
    mut commands: Commands,
    buildings: Query<(Entity, &SceneBuildingIndicator, &Building)>,
    asset_server: Res<AssetServer>,
) {
    for event in network_events.read() {
        if let ServerMessages::BuildingUpdate(BuildingUpdate { indicator, update }) = &event.message
        {
            for (entity, other_indicator, building) in buildings.iter() {
                if indicator.ne(other_indicator) {
                    continue;
                }

                let texture = match update {
                    UpdateType::Status { new_status } => building_texture(&building, *new_status),
                    UpdateType::Upgrade { upgraded_building } => {
                        println!("Updating upgraded building: {:?}", upgraded_building);
                        commands
                            .entity(entity)
                            .insert(building_collider(upgraded_building));
                        building_texture(&upgraded_building, BuildStatus::Built)
                    }
                };
                let image = asset_server.load::<Image>(texture);
                commands.entity(entity).insert(Sprite {
                    image,
                    flip_x: building_flipped(indicator),
                    ..default()
                });
            }
        }
    }
}

fn building_flipped(indicator: &SceneBuildingIndicator) -> bool {
    match indicator {
        SceneBuildingIndicator::Base(base_scene_indicator) => match base_scene_indicator {
            BaseSceneIndicator::RightWall => true,
            _ => false,
        },
        SceneBuildingIndicator::Fight(fight_scene_indicator) => match fight_scene_indicator {
            FightSceneIndicator::LeftRightWall | FightSceneIndicator::RightRightWall => true,
            _ => false,
        },
        SceneBuildingIndicator::Camp(_camp_scene_indicator) => false,
    }
}

fn building_texture(building_type: &Building, status: BuildStatus) -> &str {
    match status {
        BuildStatus::Marker => match building_type {
            Building::MainBuilding { level: _ } => "sprites/buildings/main_house_blue.png",
            Building::Archer => "sprites/buildings/archer_plot.png",
            Building::Warrior => "sprites/buildings/warrior_plot.png",
            Building::Pikeman => "sprites/buildings/pike_man_plot.png",
            Building::Wall { level: _ } => "sprites/buildings/wall_plot.png",
            Building::Tower => "",
            Building::GoldFarm => "sprites/buildings/warrior_plot.png",
        },
        BuildStatus::Built => match building_type {
            Building::MainBuilding { level } => match level {
                MainBuildingLevels::Tent => "sprites/buildings/main_house_blue.png",
                MainBuildingLevels::Hall => "sprites/buildings/main_hall.png",
                MainBuildingLevels::Castle => "sprites/buildings/main_castle.png",
            },
            Building::Archer => "sprites/buildings/archer_house.png",
            Building::Warrior => "sprites/buildings/warrior_house.png",
            Building::Pikeman => "sprites/buildings/pike_man_house.png",
            Building::Wall { level } => match level {
                WallLevels::Basic => "sprites/buildings/wall_1.png",
                WallLevels::Wood => "sprites/buildings/wall_2.png",
                WallLevels::Tower => "sprites/buildings/wall_3.png",
            },
            Building::Tower => "sprites/buildings/archer_house.png",
            Building::GoldFarm => "sprites/buildings/warrior_house.png",
        },
        BuildStatus::Destroyed => todo!(),
    }
}
