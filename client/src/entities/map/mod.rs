use bevy::prelude::*;

use bevy::color::palettes::css::{BLUE, RED, YELLOW};

use bevy::sprite::Mesh2dHandle;
use shared::map::scenes::base::BaseScene;
use shared::map::scenes::fight::FightScene;
use shared::networking::{SpawnFlag, SpawnPlayer, SpawnProjectile, SpawnUnit};
use shared::GameState;
use shared::{map::GameSceneType, networking::ServerMessages};

use crate::networking::{Connected, NetworkEvent};

use super::PartOfScene;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (load_game_scene)
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
) {
    for event in network_events.read() {
        if let ServerMessages::LoadGameScene {
            game_scene_type: map_type,
            players,
            flag,
            units,
            projectiles,
        } = &event.message
        {
            println!("Loading map {:?}...", map_type);

            for entity in entities.iter() {
                commands.entity(entity).despawn();
            }

            match map_type {
                GameSceneType::Fight => {
                    let fight = FightScene::new();
                    commands.spawn((
                        fight.left_main_building,
                        (
                            Mesh2dHandle(
                                meshes
                                    .add(Rectangle::from_size(fight.left_main_building.collider.0)),
                            ),
                            materials.add(Color::from(RED)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.left_archer_building,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(
                                    fight.left_archer_building.collider.0,
                                )),
                            ),
                            materials.add(Color::from(RED)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.left_warrior_building,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(
                                    fight.left_warrior_building.collider.0,
                                )),
                            ),
                            materials.add(Color::from(RED)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.left_pikeman_building,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(
                                    fight.left_pikeman_building.collider.0,
                                )),
                            ),
                            materials.add(Color::from(RED)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.left_left_wall,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(fight.left_left_wall.collider.0)),
                            ),
                            materials.add(Color::from(RED)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.left_right_wall,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(fight.left_right_wall.collider.0)),
                            ),
                            materials.add(Color::from(RED)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.left_gold_farm,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(fight.left_gold_farm.collider.0)),
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
                        fight.right_main_building,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(
                                    fight.right_main_building.collider.0,
                                )),
                            ),
                            materials.add(Color::from(BLUE)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.right_archer_building,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(
                                    fight.right_archer_building.collider.0,
                                )),
                            ),
                            materials.add(Color::from(BLUE)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.right_warrior_building,
                        (
                            Mesh2dHandle(meshes.add(Rectangle::from_size(
                                fight.right_warrior_building.collider.0,
                            ))),
                            materials.add(Color::from(BLUE)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.right_pikeman_building,
                        (
                            Mesh2dHandle(meshes.add(Rectangle::from_size(
                                fight.right_pikeman_building.collider.0,
                            ))),
                            materials.add(Color::from(BLUE)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.right_left_wall,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(fight.right_right_wall.collider.0)),
                            ),
                            materials.add(Color::from(BLUE)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.right_right_wall,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(fight.right_right_wall.collider.0)),
                            ),
                            materials.add(Color::from(BLUE)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        fight.right_gold_farm,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(fight.right_gold_farm.collider.0)),
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
                GameSceneType::Base(color) => {
                    let base = BaseScene::new();
                    commands.spawn((
                        base.main_building,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(base.main_building.collider.0)),
                            ),
                            materials.add(*color),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        base.archer_building,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(base.archer_building.collider.0)),
                            ),
                            materials.add(*color),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        base.warrior_building,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(base.warrior_building.collider.0)),
                            ),
                            materials.add(*color),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        base.pikeman_building,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(base.pikeman_building.collider.0)),
                            ),
                            materials.add(*color),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        base.left_wall,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(base.left_wall.collider.0)),
                            ),
                            materials.add(*color),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        base.right_wall,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(base.right_wall.collider.0)),
                            ),
                            materials.add(*color),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        base.left_gold_farm,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(base.left_gold_farm.collider.0)),
                            ),
                            materials.add(Color::srgb(212., 215., 0.)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        base.right_gold_farm,
                        (
                            Mesh2dHandle(
                                meshes.add(Rectangle::from_size(base.right_gold_farm.collider.0)),
                            ),
                            materials.add(Color::srgb(212., 215., 0.)),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ),
                        PartOfScene,
                    ));
                    commands.spawn((
                        base.left_spawn_point,
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
