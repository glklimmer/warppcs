use bevy::prelude::*;

use army::{
    ArmyFlagAssignments, ArmyFormation, ArmyPosition,
    commander::{BASE_FORMATION_OFFSET, BASE_FORMATION_WIDTH},
    flag::{Flag, FlagAssignment, FlagHolder},
    slot::ArmySlot,
};
use bevy::sprite::Anchor;
use bevy_replicon::prelude::{AppRuleExt, Replicated, SendMode, ServerTriggerExt, ToClients};
use health::Health;
use interaction::{Interactable, InteractableSound, InteractionTriggeredEvent, InteractionType};
use inventory::Inventory;
use items::{CalculatedStats, Effect, Item, ItemType};
use lobby::PlayerColor;
use physics::{
    attachment::AttachedTo,
    movement::{BoxCollider, Speed},
};
use serde::{Deserialize, Serialize};
use shared::{
    GameSceneId, Owner, Vec3LayerExt,
    enum_map::{EnumIter, EnumMap},
    map::Layers,
};
use units::{Damage, MeleeRange, ProjectileRange, Sight, Unit, UnitType};

use crate::{
    BuildStatus, Building, marker_collider, recruiting::animations::RecruitAnimationPlugin,
};

use super::item_assignment::ItemAssignment;

pub(crate) mod animations;

pub(crate) struct RecruitingPlugins;

impl Plugin for RecruitingPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(RecruitAnimationPlugin)
            .replicate_bundle::<(RecruitBuilding, Transform)>()
            .add_observer(recruit_units)
            .add_observer(recruit_commander)
            .add_systems(
                FixedUpdate,
                check_recruit.run_if(on_message::<InteractionTriggeredEvent>),
            );
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
#[require(
    Replicated,
    Transform,
    BoxCollider = marker_collider(),
    Sprite,
    Anchor::BOTTOM_CENTER,
    BuildStatus = BuildStatus::Marker,
)]
pub struct RecruitBuilding;

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

#[derive(Component)]
pub struct Recruiter;

fn recruit_units(
    trigger: On<RecruitEvent>,
    mut player_query: Query<(&Transform, &mut Inventory, &PlayerColor, &GameSceneId)>,
    mut commands: Commands,
) -> Result {
    let RecruitEvent {
        player,
        unit_type,
        items,
        original_building,
    } = &*trigger;

    if let UnitType::Commander = unit_type {
        return Ok(());
    }
    // No items when commander, but already handled
    let Some(items) = items else {
        return Ok(());
    };

    let player = *player;
    let unit_type = *unit_type;

    let (player_transform, mut inventory, color, game_scene_id) = player_query.get_mut(player)?;
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
        message: InteractableSound {
            kind: InteractionType::Recruit,
            spatial_position: player_transform.translation,
        },
    });
    Ok(())
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
        ));
    }
}

pub(crate) fn unit_stats(
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
    let time = 60. / items.calculated(Effect::AttackSpeed);
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

fn recruit_commander(
    trigger: On<RecruitEvent>,
    mut player_query: Query<(&Transform, &mut Inventory, &PlayerColor, &GameSceneId)>,
    mut commands: Commands,
) -> Result {
    let RecruitEvent {
        player,
        unit_type,
        items: _,
        original_building,
    } = &*trigger;

    let UnitType::Commander = unit_type else {
        return Ok(());
    };

    let player = *player;
    let (player_transform, mut inventory, color, game_scene_id) = player_query.get_mut(player)?;
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

    let hitpoints = 300.;
    let health = Health { hitpoints };

    let movement_speed = 35.;
    let speed = Speed(movement_speed);

    let damage = 20.;
    let damage = Damage(damage);

    let range = 10.;
    let range = MeleeRange(range);

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
                *game_scene_id,
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
        message: InteractableSound {
            kind: InteractionType::Recruit,
            spatial_position: player_transform.translation,
        },
    });
    Ok(())
}

fn check_recruit(
    mut interactions: MessageReader<InteractionTriggeredEvent>,
    player: Query<&Inventory>,
    building: Query<(&Building, Option<&ItemAssignment>), With<RecruitBuilding>>,
    mut commands: Commands,
) -> Result {
    for event in interactions.read() {
        let InteractionType::Recruit = &event.interaction else {
            continue;
        };
        let inventory = player.get(event.player)?;
        let (building, item_assignment) = building.get(event.interactable)?;

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
    Ok(())
}
