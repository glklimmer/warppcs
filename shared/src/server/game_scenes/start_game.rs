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
        entities::{Damage, Range, Unit, health::Health},
        game_scenes::travel::{SceneEnd, TravelDestinationOffset},
        physics::movement::{Speed, Velocity},
        players::{
            chest::Chest,
            interaction::{Interactable, InteractionType},
            items::{Item, ItemType, Rarity},
            mount::Mount,
        },
    },
};

use super::{
    map::{GameScene, LoadMap, SceneType},
    travel::TravelDestination,
};

pub struct StartGamePlugin;

impl Plugin for StartGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(start_game);
    }
}

fn start_game(
    trigger: Trigger<LoadMap>,
    mut players: Query<(&mut Transform, &Player), With<Player>>,
    mut commands: Commands,
) {
    let map = &**trigger.event();
    for (i, node) in map.node_references() {
        let offset = Vec3::new(10000. * i.index() as f32, 0., 0.);
        match node.scene {
            SceneType::Player {
                player,
                left,
                right,
            } => {
                let (mut transform, Player { color }) = players.get_mut(player).unwrap();
                transform.translation = offset.with_z(Layers::Player.as_f32());

                player_base(commands.reborrow(), offset, player, *color, left, right);

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
                    ));
                }
            }
            SceneType::Traversal { left, right } => camp(commands.reborrow(), offset, left, right),
        };
    }

    for edge in map.edge_references() {
        let a = map[edge.source()];
        let b = map[edge.target()];

        connect_scenes(commands.reborrow(), a, b);
    }
}

fn connect_scenes(mut commands: Commands, left: GameScene, right: GameScene) {
    let left_entity = match left.scene {
        SceneType::Player {
            player: _,
            left,
            right: _,
        } => left,
        SceneType::Traversal { left, right: _ } => left,
    };
    let right_entity = match right.scene {
        SceneType::Player {
            player: _,
            left: _,
            right,
        } => right,
        SceneType::Traversal { left: _, right } => right,
    };

    commands.entity(left_entity).insert((
        left,
        TravelDestination::new(right_entity),
        TravelDestinationOffset(50.),
    ));
    commands.entity(right_entity).insert((
        right,
        TravelDestination::new(left_entity),
        TravelDestinationOffset(-50.),
    ));
}

fn camp(
    mut commands: Commands,
    offset: Vec3,
    camp_left_scene_end: Entity,
    camp_right_scene_end: Entity,
) {
    for i in 1..10 {
        commands.spawn((
            Owner::Bandits,
            Unit {
                unit_type: UnitType::Bandit,
                swing_timer: Timer::from_seconds(4., TimerMode::Once),
                color: PlayerColor::default(),
            },
            BanditBehaviour::default(),
            Health { hitpoints: 25. },
            Range(10.),
            Speed(30.),
            Damage(10.),
            offset
                .offset_x(50. - 10. * i as f32)
                .with_layer(Layers::Unit),
        ));
    }
    commands.entity(camp_left_scene_end).insert((
        SceneEnd,
        offset
            .offset_x(-300.)
            .offset_y(-2.)
            .with_layer(Layers::Wall),
    ));
    commands.entity(camp_right_scene_end).insert((
        SceneEnd,
        offset.offset_x(300.).offset_y(-2.).with_layer(Layers::Wall),
    ));
}

fn player_base(
    mut commands: Commands,
    offset: Vec3,
    player: Entity,
    color: PlayerColor,
    left_scene_end: Entity,
    right_scene_end: Entity,
) {
    let owner = Owner::Player(player);
    commands.spawn((
        Building {
            building_type: BuildingType::MainBuilding {
                level: MainBuildingLevels::Tent,
            },
            color,
        },
        Health { hitpoints: 200. },
        BuildingType::MainBuilding {
            level: MainBuildingLevels::Tent,
        }
        .collider(),
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
    ));
    commands.spawn((
        Mount {
            mount_type: MountType::Horse,
        },
        offset.offset_x(50.).with_layer(Layers::Mount),
    ));
    commands.spawn((
        Chest::Normal,
        offset.offset_x(-50.).with_layer(Layers::Chest),
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
    ));
    commands.spawn((
        Building {
            building_type: BuildingType::Wall {
                level: WallLevels::Basic,
            },
            color,
        },
        offset.offset_x(-345.).with_layer(Layers::Wall),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(player),
        },
    ));
    commands.spawn((
        Building {
            building_type: BuildingType::GoldFarm,
            color,
        },
        offset.offset_x(320.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(player),
        },
    ));
    commands.spawn((
        Building {
            building_type: BuildingType::GoldFarm,
            color,
        },
        offset.offset_x(-265.).with_layer(Layers::Building),
        owner,
        Interactable {
            kind: InteractionType::Building,
            restricted_to: Some(player),
        },
    ));

    commands.entity(left_scene_end).insert((
        SceneEnd,
        offset
            .offset_x(-600.)
            .offset_y(-2.)
            .with_layer(Layers::Wall),
    ));
    commands.entity(right_scene_end).insert((
        SceneEnd,
        offset.offset_x(600.).offset_y(-2.).with_layer(Layers::Wall),
    ));
}
