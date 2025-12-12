use bevy::prelude::*;

use petgraph::visit::{EdgeRef, IntoNodeReferences};
use shared::{
    GameSceneId, Owner, Player, PlayerColor, SceneType, Vec3LayerExt,
    map::{
        Layers,
        buildings::{
            BuildStatus, Building, BuildingType, HealthIndicator, MainBuildingLevels,
            RecruitBuilding, WallLevels,
        },
    },
    networking::{MountType, UnitType, WorldDirection},
    server::{
        ai::BanditBehaviour,
        buildings::{gold_farm::GoldFarmTimer, item_assignment::ItemAssignment},
        entities::{Damage, MeleeRange, Unit, health::Health},
        physics::{
            collider_trigger::ColliderTrigger,
            movement::{NoWalkZone, Speed, Velocity},
        },
        players::{
            chest::Chest,
            interaction::{Interactable, InteractionType},
            items::{Item, ItemType, Rarity},
            mount::Mount,
        },
    },
};
use travel::{SceneEnd, TravelDestinationOffset, TravelDestinations};

use super::world::InitWorld;

pub struct StartGamePlugin;

impl Plugin for StartGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_world);
    }
}

fn init_world(
    init_world: On<InitWorld>,
    mut players: Query<(&mut Transform, &Player)>,
    mut commands: Commands,
) -> Result {
    let world = &**init_world.event();

    for (i, node) in world.node_references() {
        let offset = Vec3::new(10000. * i.index() as f32, 0., 0.);
        let game_scene_id = node.id;

        match node.scene {
            SceneType::Player { player, exit } => {
                let (mut transform, Player { color, .. }) = players.get_mut(player)?;
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

        let destinations = world
            .edges(i)
            .map(|edge| world[edge.target()].entry_entity())
            .collect::<Vec<_>>();

        let scene_ends = match node.scene {
            SceneType::Player { exit, .. } => vec![exit],
            SceneType::Camp { left, right } => vec![left, right],
            SceneType::Meadow { left, right } => vec![left, right],
        };

        for end in scene_ends {
            commands
                .entity(end)
                .insert((*node, TravelDestinations::new(destinations.clone())));
        }
    }

    Ok(())
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
        TravelDestinationOffset::non_player(),
        offset
            .offset_x(-400.)
            .offset_y(-2.)
            .with_layer(Layers::Wall),
        game_scene_id,
    ));
    commands.spawn((
        NoWalkZone::to_the(WorldDirection::Left),
        Transform::from_translation(offset.offset_x(-410.)),
        game_scene_id,
    ));
    commands.entity(right_scene_end).insert((
        SceneEnd,
        ColliderTrigger::Travel,
        offset.offset_x(400.).offset_y(-2.).with_layer(Layers::Wall),
        game_scene_id,
    ));
    commands.spawn((
        NoWalkZone::to_the(WorldDirection::Right),
        Transform::from_translation(offset.offset_x(410.)),
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
        TravelDestinationOffset::non_player(),
        offset
            .offset_x(-500.)
            .offset_y(-2.)
            .with_layer(Layers::Wall),
        game_scene_id,
    ));
    commands.spawn((
        NoWalkZone::to_the(WorldDirection::Left),
        Transform::from_translation(offset.offset_x(-510.)),
        game_scene_id,
    ));
    commands.entity(right_scene_end).insert((
        SceneEnd,
        ColliderTrigger::Travel,
        offset.offset_x(500.).offset_y(-2.).with_layer(Layers::Wall),
        game_scene_id,
    ));
    commands.spawn((
        NoWalkZone::to_the(WorldDirection::Right),
        Transform::from_translation(offset.offset_x(510.)),
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
        GoldFarmTimer::default(),
        offset.offset_x(-265.).with_layer(Layers::Building),
        owner,
        game_scene_id,
    ));
    commands.spawn((
        NoWalkZone::to_the(WorldDirection::Left),
        Transform::from_translation(offset.offset_x(-300.)),
        game_scene_id,
    ));
    commands.entity(exit).insert((
        SceneEnd,
        ColliderTrigger::Travel,
        TravelDestinationOffset::player(),
        offset.offset_x(700.).offset_y(-2.).with_layer(Layers::Wall),
        game_scene_id,
    ));
    commands.spawn((
        NoWalkZone::to_the(WorldDirection::Right),
        Transform::from_translation(offset.offset_x(710.)),
        game_scene_id,
    ));
}
