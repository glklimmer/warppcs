use bevy::prelude::*;

use petgraph::visit::{EdgeRef, IntoNodeReferences};

use crate::{
    Owner, Player, PlayerColor, Vec3LayerExt,
    map::{
        Layers,
        buildings::{
            BuildStatus, Building, BuildingType, HealthIndicator, MainBuildingLevels,
            RecruitBuilding, WallLevels,
        },
    },
    networking::{MountType, UnitType},
    server::{
        ai::BanditBehaviour,
        buildings::item_assignment::ItemAssignment,
        entities::{Damage, MeleeRange, Unit, health::Health},
        game_scenes::{
            GameSceneId,
            travel::{SceneEnd, TravelDestination, TravelDestinationOffset},
        },
        physics::movement::{Speed, Velocity},
        players::{
            chest::Chest,
            interaction::{Interactable, InteractionType},
            items::{Item, ItemType, Rarity},
            mount::Mount,
        },
    },
};

use super::world::{ExitType, GameScene, InitWorld, SceneType};

pub struct StartGamePlugin;

impl Plugin for StartGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_world);
    }
}

fn init_world(
    load_map: Trigger<InitWorld>,
    mut players: Query<(&mut Transform, &Player)>,
    mut commands: Commands,
) -> Result {
    let map = &**load_map.event();
    for (i, node) in map.node_references() {
        let offset = Vec3::new(10000. * i.index() as f32, 0., 0.);
        let game_scene_id = GameSceneId(i.index() + 1);

        match node.scene {
            SceneType::Player { player, exit } => {
                let (mut transform, Player { color }) = players.get_mut(player)?;
                transform.translation = offset.with_z(Layers::Player.as_f32());
                commands.entity(player).insert((
                    Owner::Player(player),
                    Health { hitpoints: 200. },
                    game_scene_id,
                ));

                player_base(
                    commands.reborrow(),
                    offset,
                    player,
                    *color,
                    exit,
                    game_scene_id,
                );

                for item_type in ItemType::all_variants() {
                    let translation = transform.translation;
                    let item = Item::builder()
                        .with_rarity(Rarity::Common)
                        .with_type(item_type)
                        .build();

                    commands.spawn((
                        item.collider(),
                        item,
                        translation.with_y(12.5).with_layer(Layers::Item),
                        Velocity(Vec2::new((fastrand::f32() - 0.5) * 100., 100.)),
                        game_scene_id,
                    ));
                }
            }
            SceneType::Camp { left, right } => {
                camp(commands.reborrow(), offset, left, right, game_scene_id)
            }
            SceneType::Meadow { left, right } => {
                meadow(commands.reborrow(), offset, left, right, game_scene_id)
            }
        };
    }

    for edge in map.edge_references() {
        let scene_a = map[edge.source()];
        let scene_b = map[edge.target()];
        let connection = edge.weight();

        connect_scenes(commands.reborrow(), scene_a, scene_b, *connection);
    }
    Ok(())
}

fn connect_scenes(
    mut commands: Commands,
    scene_a: GameScene,
    scene_b: GameScene,
    (type_a, type_b): (ExitType, ExitType),
) {
    fn get_entity_for_exit(scene: GameScene, exit_type: ExitType) -> Entity {
        match (scene.scene, exit_type) {
            (SceneType::Player { exit, .. }, ExitType::Left) => exit,
            (SceneType::Player { exit, .. }, ExitType::Right) => exit,
            (SceneType::Camp { left, .. }, ExitType::Left) => left,
            (SceneType::Camp { right, .. }, ExitType::Right) => right,
            (SceneType::Meadow { left, .. }, ExitType::Left) => left,
            (SceneType::Meadow { right, .. }, ExitType::Right) => right,
        }
    }

    let entity_a = get_entity_for_exit(scene_a, type_a);
    let entity_b = get_entity_for_exit(scene_b, type_b);

    commands.entity(entity_a).insert((
        scene_a,
        TravelDestination::new(entity_b),
        TravelDestinationOffset::from(type_a),
    ));
    commands.entity(entity_b).insert((
        scene_b,
        TravelDestination::new(entity_a),
        TravelDestinationOffset::from(type_b),
    ));
}

fn meadow(
    mut commands: Commands,
    offset: Vec3,
    left_scene_end: Entity,
    right_scene_end: Entity,
    game_scene_id: GameSceneId,
) {
    commands.spawn((
        Chest::Normal,
        offset.offset_x(-45.).with_layer(Layers::Chest),
        game_scene_id,
    ));
    commands.spawn((
        Chest::Normal,
        offset.offset_x(-15.).with_layer(Layers::Chest),
        game_scene_id,
    ));
    commands.spawn((
        Mount {
            mount_type: MountType::Horse,
        },
        offset.with_layer(Layers::Mount),
        game_scene_id,
    ));
    commands.spawn((
        Chest::Normal,
        offset.offset_x(15.).with_layer(Layers::Chest),
        game_scene_id,
    ));
    commands.spawn((
        Chest::Normal,
        offset.offset_x(45.).with_layer(Layers::Chest),
        game_scene_id,
    ));
    for i in 1..30 {
        commands.spawn((
            Owner::Bandits,
            Unit {
                unit_type: UnitType::Bandit,
                swing_timer: Timer::from_seconds(5., TimerMode::Once),
                color: PlayerColor::default(),
            },
            BanditBehaviour::default(),
            Health { hitpoints: 25. },
            MeleeRange(10.),
            Speed(30.),
            Damage(10.),
            offset
                .offset_x(150. - 10. * i as f32)
                .with_layer(Layers::Unit),
            game_scene_id,
        ));
    }
    commands.entity(left_scene_end).insert((
        SceneEnd,
        offset
            .offset_x(-400.)
            .offset_y(-2.)
            .with_layer(Layers::Wall),
        game_scene_id,
    ));
    commands.entity(right_scene_end).insert((
        SceneEnd,
        offset.offset_x(400.).offset_y(-2.).with_layer(Layers::Wall),
        game_scene_id,
    ));
}

fn camp(
    mut commands: Commands,
    offset: Vec3,
    left_scene_end: Entity,
    right_scene_end: Entity,
    game_scene_id: GameSceneId,
) {
    commands.spawn((
        Chest::Normal,
        offset.with_layer(Layers::Chest),
        game_scene_id,
    ));
    for i in 1..10 {
        commands.spawn((
            Owner::Bandits,
            Unit {
                unit_type: UnitType::Bandit,
                swing_timer: Timer::from_seconds(5., TimerMode::Once),
                color: PlayerColor::default(),
            },
            BanditBehaviour::default(),
            Health { hitpoints: 25. },
            MeleeRange(10.),
            Speed(30.),
            Damage(10.),
            offset.offset_x(-10. * i as f32).with_layer(Layers::Unit),
            game_scene_id,
        ));
    }
    commands.entity(left_scene_end).insert((
        SceneEnd,
        offset
            .offset_x(-500.)
            .offset_y(-2.)
            .with_layer(Layers::Wall),
        game_scene_id,
    ));
    commands.entity(right_scene_end).insert((
        SceneEnd,
        offset.offset_x(500.).offset_y(-2.).with_layer(Layers::Wall),
        game_scene_id,
    ));
}

fn player_base(
    mut commands: Commands,
    offset: Vec3,
    player: Entity,
    color: PlayerColor,
    exit: Entity,
    game_scene_id: GameSceneId,
) {
    let owner = Owner::Player(player);
    let main_building = Building {
        building_type: BuildingType::MainBuilding {
            level: MainBuildingLevels::Tent,
        },
        color,
    };
    commands.spawn((
        main_building,
        main_building.collider(),
        main_building.health(),
        BuildStatus::Built {
            indicator: HealthIndicator::Healthy,
        },
        offset.with_layer(Layers::Building),
        owner,
        RecruitBuilding,
        Interactable {
            kind: InteractionType::Recruit,
            restricted_to: Some(player),
        },
        game_scene_id,
    ));
    commands.spawn((
        RecruitBuilding,
        ItemAssignment::default(),
        offset.offset_x(135.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::ItemAssignment,
            restricted_to: Some(player),
        },
        game_scene_id,
    ));
    commands.spawn((
        RecruitBuilding,
        ItemAssignment::default(),
        offset.offset_x(-135.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::ItemAssignment,
            restricted_to: Some(player),
        },
        game_scene_id,
    ));
    commands.spawn((
        RecruitBuilding,
        ItemAssignment::default(),
        offset.offset_x(235.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::ItemAssignment,
            restricted_to: Some(player),
        },
        game_scene_id,
    ));
    commands.spawn((
        Building {
            building_type: BuildingType::Wall {
                level: WallLevels::Basic,
            },
            color,
        },
        offset.offset_x(390.).with_layer(Layers::Wall),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(player),
        },
        game_scene_id,
    ));
    let gold_building = Building {
        building_type: BuildingType::GoldFarm,
        color,
    };
    commands.spawn((
        gold_building,
        gold_building.collider(),
        gold_building.health(),
        BuildStatus::Built {
            indicator: HealthIndicator::Healthy,
        },
        offset.offset_x(-265.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(player),
        },
        game_scene_id,
    ));

    commands.entity(exit).insert((
        SceneEnd,
        offset
            .offset_x(-700.)
            .offset_y(-2.)
            .with_layer(Layers::Wall),
        game_scene_id,
    ));
}
