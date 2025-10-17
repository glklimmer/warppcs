use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{Replicated, SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, Owner, Player, PlayerColor, Vec3LayerExt,
    enum_map::{EnumIter, EnumMap},
    flag_collider,
    map::{
        Layers,
        buildings::{Building, RecruitBuilding},
    },
    networking::{Inventory, UnitType},
    server::{
        ai::{FollowOffset, UnitBehaviour},
        entities::{
            Damage, MeleeRange, ProjectileRange, Sight, Unit,
            commander::{
                ArmyFlagAssignments, ArmyFormation, ArmyPosition, BASE_FORMATION_OFFSET,
                BASE_FORMATION_WIDTH,
            },
            health::Health,
        },
        game_scenes::GameSceneId,
        physics::{
            army_slot::ArmySlot,
            attachment::AttachedTo,
            movement::{Speed, Velocity},
        },
        players::{
            interaction::{
                Interactable, InteractableSound, InteractionTriggeredEvent, InteractionType,
            },
            items::{CalculatedStats, Effect, Item, ItemType},
        },
    },
};

use super::item_assignment::ItemAssignment;

#[derive(Component, Deserialize, Serialize, Debug)]
#[require(
    Replicated,
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    BoxCollider = flag_collider(),
    Transform = Transform {translation: Vec3::new(0., 0., Layers::Flag.as_f32()), ..default()}
)]
pub struct Flag {
    #[entities]
    pub original_building: Entity,
    pub unit_type: UnitType,
    pub color: PlayerColor,
}

/// This component is added on Player. Tuple entity is flag.
#[derive(Component, Clone, Copy, Deref, DerefMut, Deserialize, Serialize)]
#[require(Replicated)]
pub struct FlagHolder(#[entities] pub Entity);

#[derive(Component, Deserialize, Serialize, Deref, DerefMut)]
#[relationship(relationship_target = FlagUnits)]
pub struct FlagAssignment(#[entities] pub Entity);

#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
#[relationship_target(relationship = FlagAssignment)]
pub struct FlagUnits(#[entities] Vec<Entity>);

pub fn assign_offset(
    trigger: Trigger<OnAdd, FlagAssignment>,
    mut units: Query<&mut FollowOffset>,
    flag_units_query: Query<&FlagUnits>,
    flag_assignment_query: Query<&FlagAssignment>,
) {
    let flag_assignment = flag_assignment_query.get(trigger.target()).unwrap();
    let flag_entity = **flag_assignment;

    let Ok(flag_units) = flag_units_query.get(flag_entity) else {
        return;
    };

    let mut unit_entities = (**flag_units).to_vec();
    unit_entities.push(trigger.target());

    fastrand::shuffle(&mut unit_entities);

    let count = unit_entities.len() as f32;
    let half = (count - 1.0) / 2.0;
    let spacing = 15.0;
    let shift = if unit_entities.len() % 2 == 1 {
        spacing / 2.0
    } else {
        0.0
    };

    for (i, unit_entity) in unit_entities.into_iter().enumerate() {
        if let Ok(mut offset) = units.get_mut(unit_entity) {
            let index = i as f32;
            let offset_x = spacing * (index - half) - shift;
            offset.0 = Vec2::new(offset_x, 0.0);
        }
    }
}

#[derive(Event, Deserialize, Serialize)]
pub struct RecruitEvent {
    player: Entity,
    unit_type: UnitType,
    items: Option<Vec<Item>>,
    original_building: Entity,
}

impl RecruitEvent {
    pub fn new(
        player: Entity,
        unit_type: UnitType,
        items: Option<Vec<Item>>,
        original_building: Entity,
    ) -> Self {
        Self {
            player,
            unit_type,
            items,
            original_building,
        }
    }
}

pub fn recruit_units(
    trigger: Trigger<RecruitEvent>,
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Inventory, &Player, &GameSceneId)>,
) {
    let RecruitEvent {
        player,
        unit_type,
        items,
        original_building,
    } = &*trigger;

    if let UnitType::Commander = unit_type {
        return;
    }
    let player = *player;
    let unit_type = *unit_type;
    let Some(items) = items else {
        return;
    };

    let (player_transform, mut inventory, Player { color }, game_scene_id) =
        player_query.get_mut(player).unwrap();
    let player_translation = player_transform.translation;

    let cost = &unit_type.recruitment_cost();
    inventory.gold -= cost.gold;

    let owner = Owner::Player(player);
    let flag_entity = commands
        .spawn((
            Flag {
                original_building: *original_building,
                unit_type,
                color: *color,
            },
            AttachedTo(player),
            Interactable {
                kind: InteractionType::Flag,
                restricted_to: Some(player),
            },
            owner,
            *game_scene_id,
        ))
        .id();

    commands.entity(player).insert(FlagHolder(flag_entity));

    spawn_units(
        commands.reborrow(),
        unit_type,
        items,
        player_translation,
        owner,
        flag_entity,
        *color,
        game_scene_id,
    );

    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        event: InteractableSound {
            kind: InteractionType::Recruit,
            spatial_position: player_transform.translation,
        },
    });
}

fn spawn_units(
    mut commands: Commands,
    unit_type: UnitType,
    items: &[Item],
    position: Vec3,
    owner: Owner,
    flag_entity: Entity,
    color: PlayerColor,
    game_scene_id: &GameSceneId,
) {
    let unit_amount = items.calculated(Effect::UnitAmount) as i32;

    let (unit, health, speed, damage, melee_range, projectile_range, sight) =
        unit_stats(unit_type, items, color);

    for _ in 1..=unit_amount {
        commands.spawn((
            position.with_layer(Layers::Unit),
            unit.clone(),
            health,
            speed,
            damage,
            melee_range,
            projectile_range,
            sight,
            owner,
            *game_scene_id,
            FlagAssignment(flag_entity),
            UnitBehaviour::default(),
        ));
    }
}

pub fn unit_stats(
    unit_type: UnitType,
    items: &[Item],
    color: PlayerColor,
) -> (
    Unit,
    Health,
    Speed,
    Damage,
    MeleeRange,
    ProjectileRange,
    Sight,
) {
    let time = items.calculated(Effect::AttackSpeed) / 2.;
    let unit = Unit {
        swing_timer: Timer::from_seconds(time, TimerMode::Once),
        unit_type,
        color,
    };

    let hitpoints = items.calculated(Effect::Health);
    let health = Health { hitpoints };

    let movement_speed = items.calculated(Effect::MovementSpeed);
    let speed = Speed(movement_speed);

    let damage = items.calculated(Effect::Damage);
    let damage = Damage(damage);

    let melee_range = items.calculated(|item: &Item| {
        let ItemType::Weapon(weapon) = item.item_type else {
            return None;
        };
        Some(Effect::MeleeRange(weapon))
    });
    let melee_range = MeleeRange(melee_range);

    let projectile_range = items.calculated(|item: &Item| {
        let ItemType::Weapon(weapon) = item.item_type else {
            return None;
        };
        Some(Effect::ProjectileRange(weapon))
    });
    let projectile_range = ProjectileRange(projectile_range);

    let sight = items.calculated(Effect::Sight);
    let sight = Sight(sight);

    (
        unit,
        health,
        speed,
        damage,
        melee_range,
        projectile_range,
        sight,
    )
}

pub fn recruit_commander(
    trigger: Trigger<RecruitEvent>,
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Inventory, &Player, &GameSceneId)>,
) {
    let RecruitEvent {
        player,
        unit_type,
        items: _,
        original_building,
    } = &*trigger;

    let UnitType::Commander = unit_type else {
        return;
    };

    let player = *player;
    let (player_transform, mut inventory, Player { color }, game_scene_id) =
        player_query.get_mut(player).unwrap();
    let player_translation = player_transform.translation;

    let cost = &unit_type.recruitment_cost();
    inventory.gold -= cost.gold;

    let owner = Owner::Player(player);
    let flag_entity = commands
        .spawn((
            Flag {
                original_building: *original_building,
                unit_type: *unit_type,
                color: *color,
            },
            AttachedTo(player),
            Interactable {
                kind: InteractionType::Flag,
                restricted_to: Some(player),
            },
            owner,
            *game_scene_id,
        ))
        .id();

    commands.entity(player).insert(FlagHolder(flag_entity));

    let time = 2.;
    let unit = Unit {
        swing_timer: Timer::from_seconds(time, TimerMode::Once),
        unit_type: UnitType::Commander,
        color: *color,
    };

    let hitpoints = 100.;
    let health = Health { hitpoints };

    let movement_speed = 35.;
    let speed = Speed(movement_speed);

    let damage = 20.;
    let damage = Damage(damage);

    let range = 10.;
    let range = MeleeRange(range);

    let offset = Vec2::new(-22., 0.);
    let commander = commands
        .spawn((
            player_translation.with_layer(Layers::Flag),
            unit.clone(),
            health,
            speed,
            damage,
            range,
            owner,
            *game_scene_id,
            FlagAssignment(flag_entity),
            FollowOffset(offset),
            UnitBehaviour::default(),
            Interactable {
                kind: InteractionType::Commander,
                restricted_to: Some(player),
            },
            ArmyFlagAssignments::default(),
        ))
        .id();

    let mut formation_offset = 0.;

    let mut army_formation: Vec<Entity> = vec![];

    ArmyPosition::all_variants().iter().for_each(|_| {
        formation_offset += (BASE_FORMATION_WIDTH) + BASE_FORMATION_OFFSET;
        let formation = commands
            .spawn((
                ArmySlot {
                    commander,
                    offset: formation_offset,
                },
                Velocity::default(),
                Transform::from_translation(Vec3::new(-formation_offset, 0., 0.))
                    .with_scale(Vec3::new(BASE_FORMATION_WIDTH, 1., 1.)),
            ))
            .id();
        army_formation.push(formation);
    });
    commands.entity(commander).insert(ArmyFormation {
        positions: EnumMap::new(|c| match c {
            ArmyPosition::Front => army_formation[0],
            ArmyPosition::Middle => army_formation[1],
            ArmyPosition::Back => army_formation[2],
        }),
    });

    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        event: InteractableSound {
            kind: InteractionType::Recruit,
            spatial_position: player_transform.translation,
        },
    });
}

pub fn check_recruit(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    player: Query<&Inventory>,
    building: Query<(&Building, Option<&ItemAssignment>), With<RecruitBuilding>>,
) {
    for event in interactions.read() {
        let InteractionType::Recruit = &event.interaction else {
            continue;
        };
        let inventory = player.get(event.player).unwrap();
        let (building, item_assignment) = building.get(event.interactable).unwrap();

        let Some(unit_type) = building.unit_type() else {
            continue;
        };

        let cost = unit_type.recruitment_cost();
        if !inventory.gold.ge(&cost.gold) {
            println!("Not enough gold for recruitment");
            continue;
        }

        let items: Option<Vec<_>> = item_assignment
            .map(|assignment| assignment.items.clone().into_iter().flatten().collect());

        commands.trigger(RecruitEvent {
            player: event.player,
            unit_type,
            items,
            original_building: event.interactable,
        });
    }
}
