use bevy::prelude::*;

use bevy::{
    app::Plugin,
    ecs::{entity::Entity, system::In, world::World},
    reflect::Map,
    remote::{BrpError, BrpResult, RemotePlugin, http::RemoteHttpPlugin},
};
use console_protocol::*;
use serde_json::{Value, json};

use crate::{
    ClientPlayerMap, Owner, Player, PlayerColor, Vec3LayerExt,
    enum_map::{EnumIter, EnumMap},
    map::{
        Layers,
        buildings::{
            BuildStatus, Building, BuildingType, HealthIndicator, RecruitBuilding, RespawnZone,
        },
    },
    networking::UnitType,
    server::{entities::commander::ArmyFormation, physics::army_slot::ArmySlot},
};

use super::{
    ai::{FollowOffset, UnitBehaviour},
    buildings::{
        item_assignment::{ItemAssignment, ItemSlot},
        recruiting::{Flag, FlagAssignment, FlagHolder, RecruitEvent},
    },
    entities::{
        Damage, Range, Unit,
        commander::{
            ArmyFlagAssignments, ArmyPosition, BASE_FORMATION_OFFSET, BASE_FORMATION_WIDTH,
        },
        health::Health,
    },
    physics::{
        attachment::AttachedTo,
        movement::{Speed, Velocity},
    },
    players::{
        interaction::{Interactable, InteractionType},
        items::{Item, ItemType, MeleeWeapon, ProjectileWeapon, Rarity, WeaponType},
    },
};

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            RemotePlugin::default()
                .with_method(BRP_SPAWN_UNIT, spawn_unit_handler)
                .with_method(BRP_SPAWN_RANDOM_ITEM, spawn_random_items)
                .with_method(BRP_SPAWN_FULL_COMMANDER, spawn_full_commander),
            RemoteHttpPlugin::default(),
        ));
    }
}

trait PlayerCommand {
    fn player(&self) -> u8;

    fn player_entity(&self, world: &mut World) -> BrpResult<Entity> {
        let client_player_map = world
            .get_resource::<ClientPlayerMap>()
            .ok_or_else(|| BrpError::internal("Missing ClientPlayerMap resource"))?;
        let (_, player) = client_player_map
            .get_at(self.player() as usize)
            .ok_or_else(|| BrpError::internal("Player index out of bounds"))?;
        let entity = player
            .try_downcast_ref::<Entity>()
            .ok_or_else(|| BrpError::internal("Value in ClientPlayerMap wasn’t an Entity"))?;
        Ok(*entity)
    }
}

impl PlayerCommand for BrpSpawnItems {
    fn player(&self) -> u8 {
        self.player
    }
}
impl PlayerCommand for BrpSpawnUnit {
    fn player(&self) -> u8 {
        self.player
    }
}

impl PlayerCommand for BrpSpawnFullCommander {
    fn player(&self) -> u8 {
        self.player
    }
}

fn spawn_unit_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult<Value> {
    let value = params.ok_or_else(|| BrpError::internal("spawn-units requires parameters"))?;

    let unit_req: BrpSpawnUnit = serde_json::from_value(value)
        .map_err(|e| BrpError::internal(format!("invalid spawn parameters: {e}")))?;

    let unit_type = match unit_req.unit.as_str() {
        "archer" => UnitType::Archer,
        "pikemen" => UnitType::Pikeman,
        "shield" => UnitType::Shieldwarrior,
        other => {
            return Err(BrpError::internal(format!("unknown unit type `{other}`")));
        }
    };

    let player_entity = unit_req.player_entity(world)?;
    let (player_transform, player) = {
        let mut query: QueryState<(&Transform, &Player)> = QueryState::new(world);
        query.get(world, player_entity).unwrap()
    };

    let weapon_type = match unit_type {
        UnitType::Shieldwarrior => ItemType::Weapon(WeaponType::Melee(MeleeWeapon::SwordAndShield)),
        UnitType::Pikeman => ItemType::Weapon(WeaponType::Melee(MeleeWeapon::Pike)),
        UnitType::Archer => ItemType::Weapon(WeaponType::Projectile(ProjectileWeapon::Bow)),
        UnitType::Bandit => todo!(),
        UnitType::Commander => todo!(),
    };

    let weapon = Item::builder().with_type(weapon_type).build();
    let head = Item::builder().with_type(ItemType::Head).build();
    let chest = Item::builder().with_type(ItemType::Chest).build();
    let feet = Item::builder().with_type(ItemType::Feet).build();

    let building = world
        .spawn((
            Building {
                building_type: BuildingType::Unit { weapon: unit_type },
                color: player.color,
            },
            RecruitBuilding,
            RespawnZone::default(),
            ItemAssignment {
                items: EnumMap::new(|c| match c {
                    ItemSlot::Weapon => Some(weapon.clone()),
                    ItemSlot::Chest => Some(chest.clone()),
                    ItemSlot::Head => Some(head.clone()),
                    ItemSlot::Feet => Some(feet.clone()),
                }),
            },
            BuildStatus::Built {
                indicator: HealthIndicator::Healthy,
            },
            player_transform.translation.with_layer(Layers::Building),
            Owner::Player(player_entity),
        ))
        .id();

    world.trigger(RecruitEvent::new(
        player_entity,
        unit_type,
        Some(vec![weapon, head, chest, feet]),
        building,
    ));

    Ok(json!("success"))
}

fn spawn_random_items(In(params): In<Option<serde_json::Value>>, world: &mut World) -> BrpResult {
    if let Some(value) = params {
        if let Ok(brp) = serde_json::from_value::<BrpSpawnItems>(value) {
            let player_entity = brp.player_entity(world)?;

            let player_pos = {
                let mut query: QueryState<&Transform> = QueryState::new(world);
                let transform = query.get(world, player_entity).unwrap();
                transform.translation
            };

            for item_type in ItemType::all_variants() {
                let item = Item::builder()
                    .with_rarity(Rarity::Common)
                    .with_type(item_type)
                    .build();

                world.spawn((
                    item.collider(),
                    item,
                    player_pos.with_y(12.5).with_layer(Layers::Item),
                    Velocity(Vec2::new((fastrand::f32() - 0.5) * 100., 100.)),
                ));
            }
        }
    }

    Ok(json!("success"))
}

fn spawn_full_commander(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let value =
        params.ok_or_else(|| BrpError::internal("spawn-full-commander requires parameters"))?;

    let brp: BrpSpawnFullCommander = serde_json::from_value(value)
        .map_err(|e| BrpError::internal(format!("invalid commander parameters: {e}")))?;
    let player = brp.player_entity(world)?;
    let color = world.entity(player).get::<Player>().unwrap().color;

    let owner = Owner::Player(player);
    let flag_commander = world
        .spawn((
            Flag {
                original_building: player,
                unit_type: UnitType::Commander,
                color,
            },
            AttachedTo(player),
            Interactable {
                kind: InteractionType::Flag,
                restricted_to: Some(player),
            },
            owner,
        ))
        .id();

    world.entity_mut(player).insert(FlagHolder(flag_commander));
    let player_translation = world
        .query::<&Transform>()
        .get_mut(world, player)
        .unwrap()
        .translation;
    let time = 3.;
    let unit = Unit {
        swing_timer: Timer::from_seconds(time, TimerMode::Repeating),
        unit_type: UnitType::Commander,
        color,
    };

    let hitpoints = 100.;
    let health = Health { hitpoints };

    let movement_speed = 50.;
    let speed = Speed(movement_speed);

    let damage = 20.;
    let damage = Damage(damage);

    let range = 10.;
    let range = Range(range);

    let offset = Vec2::new(-18., 0.);

    let commander = world
        .spawn((
            player_translation.with_layer(Layers::Flag),
            unit.clone(),
            health,
            speed,
            damage,
            range,
            owner,
            FlagAssignment(flag_commander),
            FollowOffset(offset),
            UnitBehaviour::default(),
            Interactable {
                kind: InteractionType::Commander,
                restricted_to: Some(player),
            },
        ))
        .id();

    let mut formation_offset = 0.;

    let mut army_formation: Vec<Entity> = vec![];

    ArmyPosition::all_variants().iter().for_each(|_| {
        formation_offset += (BASE_FORMATION_WIDTH) + BASE_FORMATION_OFFSET;
        let formation = world
            .spawn((
                ArmySlot {
                    commander,
                    offset: formation_offset,
                },
                Velocity::default(),
                Transform::default(),
            ))
            .id();
        army_formation.push(formation);
    });

    let front = spawn_unit(
        world,
        player,
        army_formation[0],
        UnitType::Shieldwarrior,
        player_translation,
        color,
    );
    let middle = spawn_unit(
        world,
        player,
        army_formation[1],
        UnitType::Pikeman,
        player_translation,
        color,
    );
    let back = spawn_unit(
        world,
        player,
        army_formation[2],
        UnitType::Archer,
        player_translation,
        color,
    );

    world.entity_mut(commander).insert((
        ArmyFlagAssignments {
            flags: EnumMap::new(|c| match c {
                ArmyPosition::Front => Some(front),
                ArmyPosition::Middle => Some(middle),
                ArmyPosition::Back => Some(back),
            }),
        },
        ArmyFormation {
            positions: EnumMap::new(|c| match c {
                ArmyPosition::Front => army_formation[0],
                ArmyPosition::Middle => army_formation[1],
                ArmyPosition::Back => army_formation[2],
            }),
        },
    ));

    Ok(json!("success"))
}

fn spawn_unit(
    world: &mut World,
    player: Entity,
    commander: Entity,
    unit_type: UnitType,
    player_translation: Vec3,
    color: PlayerColor,
) -> Entity {
    let owner = Owner::Player(player);
    let flag_entity = world
        .spawn((
            Flag {
                original_building: player,
                unit_type,
                color,
            },
            AttachedTo(commander),
            owner,
            Visibility::Hidden,
        ))
        .id();

    let unit = Unit {
        swing_timer: Timer::from_seconds(1., TimerMode::Repeating),
        unit_type,
        color,
    };

    let hitpoints = 200.;
    let health = Health { hitpoints };

    let movement_speed = 40.;
    let speed = Speed(movement_speed);

    let damage = 20.;
    let damage = Damage(damage);

    let range = match unit_type {
        UnitType::Shieldwarrior => 10.,
        UnitType::Pikeman => 20.,
        UnitType::Archer => 100.,
        UnitType::Bandit => todo!(),
        UnitType::Commander => todo!(),
    };
    let range = Range(range);

    for _ in 1..=4 {
        world.spawn((
            player_translation.with_layer(Layers::Flag),
            unit.clone(),
            health,
            speed,
            damage,
            range,
            owner,
            FlagAssignment(flag_entity),
            UnitBehaviour::default(),
        ));
    }
    flag_entity
}
