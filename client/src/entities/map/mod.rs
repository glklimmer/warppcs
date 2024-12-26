use bevy::prelude::*;

use bevy::{color::palettes::css::YELLOW, sprite::Mesh2dHandle};
use shared::map::scenes::fight::FightSceneIndicator;
use shared::{
    map::{
        buildings::{BuildStatus, BuildingBundle, BuildingTextures},
        scenes::{
            base::{BaseScene, BaseSceneIndicator},
            fight::FightScene,
            SceneBuildingIndicator,
        },
        GameSceneType,
    },
    networking::{
        BuildingUpdate, ServerMessages, SpawnFlag, SpawnPlayer, SpawnProjectile, SpawnUnit,
    },
    GameState,
};

use crate::networking::{Connected, NetworkEvent};

use super::PartOfScene;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (load_game_scene, update_building)
                .run_if(on_event::<NetworkEvent>())
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
                    commands
                        .spawn((
                            fight.left_main_building,
                            SceneBuildingIndicator::Fight(FightSceneIndicator::LeftMainBuilding),
                            (
                                asset_server.load::<Image>("sprites/buildings/main_house_red.png"),
                                Sprite::default(),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ))
                        .insert(Transform {
                            scale: Vec3::splat(3.0),
                            ..fight.left_main_building.transform
                        });

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

                    commands
                        .spawn((
                            fight.right_main_building,
                            SceneBuildingIndicator::Fight(FightSceneIndicator::RightMainBuilding),
                            (
                                asset_server.load::<Image>("sprites/buildings/main_house_blue.png"),
                                Sprite::default(),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ))
                        .insert(Transform {
                            scale: Vec3::splat(3.0),
                            ..fight.right_main_building.transform
                        });
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
                    commands
                        .spawn((
                            base.main_building,
                            SceneBuildingIndicator::Base(BaseSceneIndicator::MainBuilding),
                            (
                                asset_server.load::<Image>("sprites/buildings/main_house_blue.png"),
                                Sprite::default(),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ))
                        .insert(Transform {
                            scale: Vec3::splat(3.0),
                            ..base.main_building.transform
                        });
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
                    commands
                        .spawn((
                            base.left_gold_farm,
                            SceneBuildingIndicator::Base(BaseSceneIndicator::LeftGoldFarm),
                            (
                                asset_server.load::<Image>(base.left_gold_farm.textures.marker),
                                Sprite::default(),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ))
                        .insert(Transform {
                            scale: Vec3::splat(3.0),
                            ..base.left_gold_farm.transform
                        });
                    commands
                        .spawn((
                            base.right_gold_farm,
                            SceneBuildingIndicator::Base(BaseSceneIndicator::RightGoldFarm),
                            (
                                asset_server.load::<Image>(base.right_gold_farm.textures.marker),
                                Sprite::default(),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ),
                            PartOfScene,
                        ))
                        .insert(Transform {
                            scale: Vec3::splat(3.0),
                            ..base.right_gold_farm.transform
                        });
                    commands.spawn((
                        base.left_spawn_point,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::LeftSpawnPoint),
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(base.left_spawn_point.collider.0)),
                            ),
                            materials.add(Color::from(YELLOW)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        base.right_spawn_point,
                        SceneBuildingIndicator::Base(BaseSceneIndicator::RightSpawnPoint),
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(base.left_spawn_point.collider.0)),
                            ),
                            materials.add(Color::from(YELLOW)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                }
                GameSceneType::Camp => todo!(),
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
    buildings: &[BuildingUpdate],
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    building_bundle: BuildingBundle,
    indicator: SceneBuildingIndicator,
) {
    commands
        .spawn((
            building_bundle,
            indicator,
            (
                asset_server.load::<Image>(building_texture(buildings, indicator, building_bundle)),
                Sprite::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ),
            PartOfScene,
        ))
        .insert(Transform {
            scale: Vec3::splat(3.0),
            ..building_bundle.transform
        });
}

fn building_texture(
    buildings: &[BuildingUpdate],
    indicator: SceneBuildingIndicator,
    building_bundle: BuildingBundle,
) -> String {
    let update = buildings
        .iter()
        .find(|update| update.indicator.eq(&indicator));
    let texture = match update {
        Some(update) => match update.status {
            BuildStatus::Marker => building_bundle.textures.marker,
            BuildStatus::Built => building_bundle.textures.built,
            BuildStatus::Destroyed => building_bundle.textures.marker,
        },
        None => building_bundle.textures.marker,
    };
    texture.to_string()
}

fn update_building(
    mut network_events: EventReader<NetworkEvent>,
    mut commands: Commands,
    buildings: Query<(Entity, &SceneBuildingIndicator, &BuildingTextures)>,
    asset_server: Res<AssetServer>,
) {
    for event in network_events.read() {
        if let ServerMessages::BuildingUpdate(BuildingUpdate { indicator, status }) = &event.message
        {
            for (entity, other_indicator, textures) in buildings.iter() {
                if !indicator.eq(other_indicator) {
                    continue;
                }

                let texture = match status {
                    BuildStatus::Marker => textures.marker,
                    BuildStatus::Built => textures.built,
                    BuildStatus::Destroyed => textures.marker,
                };

                let texture = asset_server.load::<Image>(texture);
                commands.entity(entity).insert(texture);
            }
        }
    }
}
