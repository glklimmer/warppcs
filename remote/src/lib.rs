use bevy::prelude::*;

use ai::{ArmyFormationTo, BanditBehaviour, UnitBehaviour, offset::FollowOffset};
use army::{
    ArmyFlagAssignments, ArmyFormation, ArmyPosition,
    commander::{BASE_FORMATION_OFFSET, BASE_FORMATION_WIDTH},
    flag::{Flag, FlagAssignment, FlagHolder},
    slot::ArmySlot,
};
use bevy::{
    app::Plugin,
    ecs::{entity::Entity, system::In, world::World},
    input::common_conditions::input_just_pressed,
    remote::{BrpError, BrpResult, RemotePlugin, http::RemoteHttpPlugin},
};
use buildings::{
    BuildStatus, Building, BuildingType, HealthIndicator,
    item_assignment::{ItemAssignment, ItemSlot},
    recruiting::{RecruitBuilding, RecruitEvent},
    respawn::RespawnZone,
};
use console_protocol::*;
use health::Health;
use interaction::{Interactable, InteractionType};
use items::{Item, ItemType, MeleeWeapon, ProjectileWeapon, Rarity, WeaponType};
use lobby::{ClientPlayerMap, PlayerColor};
use physics::{
    attachment::AttachedTo,
    movement::{Speed, Velocity},
};
use player::Player;
use serde_json::{Value, json};
use shared::{
    GameSceneId, Owner, Vec3LayerExt,
    enum_map::{EnumIter, EnumMap},
    map::Layers,
};
use units::{Damage, MeleeRange, ProjectileRange, Sight, Unit, UnitType};

pub struct CheatRemotePlugin;

impl Plugin for CheatRemotePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            RemotePlugin::default()
                .with_method(BRP_SPAWN_UNIT, spawn_unit_handler)
                .with_method(BRP_SPAWN_RANDOM_ITEM, spawn_random_items)
                .with_method(BRP_SPAWN_FULL_COMMANDER, spawn_full_commander)
                .with_method(BRP_SPAWN_UNIT_AND_BANDITS, spawn_unit_and_bandits),
            RemoteHttpPlugin::default(),
        ))
        .add_systems(Update, test.run_if(input_just_pressed(KeyCode::Space)));
    }
}

trait PlayerCommand {
    fn player(&self) -> u8;

    fn player_entity(&self, world: &mut World) -> BrpResult<Entity> {
        let client_player_map = world
            .get_resource::<ClientPlayerMap>()
            .ok_or_else(|| BrpError::internal("Missing ClientPlayerMap resource"))?;
        let (_, player) = client_player_map
            .iter()
            .nth(self.player() as usize)
            .ok_or_else(|| BrpError::internal("Player index out of bounds"))?;
        let entity = player;

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

impl PlayerCommand for BrpSpawnUnitAndBandits {
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
    let (player_transform, color, game_scene_id) = {
        let mut query: QueryState<(&Transform, &PlayerColor, &GameSceneId)> =
            QueryState::new(world);
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

    let building = Building {
        building_type: BuildingType::Unit { weapon: unit_type },
        color: *color,
    };
    let building = world
        .spawn((
            building.collider(),
            building.health(),
            building,
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
            *game_scene_id,
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
    if let Some(value) = params
        && let Ok(brp) = serde_json::from_value::<BrpSpawnItems>(value)
    {
        let player_entity = brp.player_entity(world)?;

        let (player_pos, game_scene_id) = {
            let mut query: QueryState<(&Transform, &GameSceneId)> = QueryState::new(world);
            let (transform, game_scene_id) = query.get(world, player_entity).unwrap();
            (transform.translation, *game_scene_id)
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
                game_scene_id,
            ));
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
    let (color, game_scene_id) = world
        .entity(player)
        .get_components::<(&PlayerColor, &GameSceneId)>()
        .unwrap();
    let color = *color;

    let game_scene_id = *game_scene_id;

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
    let range = MeleeRange(range);

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
            game_scene_id,
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

    let front = spawn_unit_world(
        world,
        player,
        army_formation[0],
        UnitType::Shieldwarrior,
        player_translation,
        color,
        game_scene_id,
    );
    let middle = spawn_unit_world(
        world,
        player,
        army_formation[1],
        UnitType::Pikeman,
        player_translation,
        color,
        game_scene_id,
    );
    let back = spawn_unit_world(
        world,
        player,
        army_formation[2],
        UnitType::Archer,
        player_translation,
        color,
        game_scene_id,
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

fn spawn_unit_and_bandits(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    if let Some(value) = params
        && let Ok(brp) = serde_json::from_value::<BrpSpawnUnitAndBandits>(value)
    {
        let player_entity = brp.player_entity(world)?;

        let (player_pos, game_scene_id) = {
            let mut query: QueryState<(&Transform, &GameSceneId)> = QueryState::new(world);
            let (transform, game_scene_id) = query.get(world, player_entity).unwrap();
            (transform.translation, *game_scene_id)
        };

        let weapon = Item::builder()
            .with_type(ItemType::Weapon(WeaponType::Projectile(
                ProjectileWeapon::Bow,
            )))
            .build();
        let head = Item::builder().with_type(ItemType::Head).build();
        let chest = Item::builder().with_type(ItemType::Chest).build();
        let feet = Item::builder().with_type(ItemType::Feet).build();

        for i in 1..=10 {
            world.spawn((
                Owner::Bandits,
                Unit {
                    unit_type: UnitType::Bandit,
                    swing_timer: Timer::from_seconds(5., TimerMode::Once),
                    color: PlayerColor::default(),
                },
                BanditBehaviour::default(),
                Health { hitpoints: 55. },
                MeleeRange(10.),
                Sight::default(),
                Speed(30.),
                Damage(10.),
                game_scene_id,
                player_pos
                    .offset_x(350. - 10. * i as f32)
                    .with_layer(Layers::Unit),
            ));
        }

        world.trigger(RecruitEvent::new(
            player_entity,
            UnitType::Archer,
            Some(vec![weapon, head, chest, feet]),
            Entity::PLACEHOLDER,
        ));
    }
    Ok(json!("success"))
}

fn spawn_unit(
    mut commands: Commands,
    player: Entity,
    formation: Entity,
    unit_type: UnitType,
    player_translation: Vec3,
    color: PlayerColor,
    game_scene_id: GameSceneId,
    amount: i32,
    commander: Entity,
) -> Entity {
    let owner = Owner::Player(player);
    let flag_entity = commands
        .spawn((
            Flag {
                original_building: player,
                unit_type,
                color,
            },
            AttachedTo(formation),
            owner,
            Visibility::Hidden,
            game_scene_id,
        ))
        .id();

    let unit = Unit {
        swing_timer: Timer::from_seconds(4., TimerMode::Repeating),
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
        UnitType::Archer => 10.,
        UnitType::Bandit => todo!(),
        UnitType::Commander => todo!(),
    };
    let melee_range_shield = MeleeRange(range);
    let melee_range_pike = MeleeRange(range * 2.);
    let projectile_range = ProjectileRange(range * 10.);

    for i in 1..=amount {
        let mut un = commands.spawn((
            Name::new(format!("{:?} {}", unit_type, i)),
            player_translation.with_layer(Layers::Flag),
            unit.clone(),
            health,
            speed,
            damage,
            owner,
            game_scene_id,
            FlagAssignment(flag_entity),
            UnitBehaviour::default(),
            ArmyFormationTo(commander),
        ));
        if unit.unit_type.eq(&UnitType::Shieldwarrior) {
            un.insert(melee_range_shield);
        }
        if unit.unit_type.eq(&UnitType::Pikeman) {
            un.insert(melee_range_pike);
        }
        if unit.unit_type.eq(&UnitType::Archer) {
            un.insert(projectile_range);
        }
    }
    flag_entity
}

fn test(
    mut commands: Commands,
    player: Query<(Entity, &GameSceneId, &Transform), With<Player>>,
) -> Result {
    let (player_entity, game_scene_id, player_pos) = player.single()?;

    let game_scene_id = *game_scene_id;

    for i in 1..=3 {
        commands.spawn((
            Name::new(format!("Bandit {}", i)),
            Owner::Bandits,
            Unit {
                unit_type: UnitType::Bandit,
                swing_timer: Timer::from_seconds(5., TimerMode::Once),
                color: PlayerColor::default(),
            },
            BanditBehaviour::default(),
            Health { hitpoints: 255. },
            MeleeRange(10.),
            Sight::default(),
            Speed(30.),
            Damage(10.),
            game_scene_id,
            Transform::from_xyz(350. - 10. * i as f32, 1., 1.),
        ));
    }

    let owner = Owner::Player(player_entity);
    let flag_commander = commands
        .spawn((
            Flag {
                original_building: player_entity,
                unit_type: UnitType::Commander,
                color: PlayerColor::Blue,
            },
            AttachedTo(player_entity),
            Interactable {
                kind: InteractionType::Flag,
                restricted_to: Some(player_entity),
            },
            owner,
        ))
        .id();

    commands
        .entity(player_entity)
        .insert(FlagHolder(flag_commander));
    let time = 3.;
    let unit = Unit {
        swing_timer: Timer::from_seconds(time, TimerMode::Repeating),
        unit_type: UnitType::Commander,
        color: PlayerColor::Blue,
    };

    let hitpoints = 100.;
    let health = Health { hitpoints };

    let movement_speed = 50.;
    let speed = Speed(movement_speed);

    let damage = 20.;
    let damage = Damage(damage);

    let range = 10.;
    let range = MeleeRange(range);

    let offset = Vec2::new(-18., 0.);

    let commander = commands
        .spawn((
            Name::new("Commander"),
            player_pos
                .translation
                .offset_x(-100.)
                .with_layer(Layers::Unit),
            unit.clone(),
            health,
            speed,
            damage,
            range,
            owner,
            FlagAssignment(flag_commander),
            game_scene_id,
            FollowOffset(offset),
            Sight(30.),
            UnitBehaviour::default(),
            Interactable {
                kind: InteractionType::Commander,
                restricted_to: Some(player_entity),
            },
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
                Transform::default(),
            ))
            .id();
        army_formation.push(formation);
    });

    let player_translation = player_pos.translation;
    let color = PlayerColor::Blue;
    let front = spawn_unit(
        commands.reborrow(),
        player_entity,
        army_formation[0],
        UnitType::Shieldwarrior,
        player_translation,
        color,
        game_scene_id,
        2,
        commander,
    );
    let middle = spawn_unit(
        commands.reborrow(),
        player_entity,
        army_formation[1],
        UnitType::Pikeman,
        player_translation,
        color,
        game_scene_id,
        1,
        commander,
    );
    let back = spawn_unit(
        commands.reborrow(),
        player_entity,
        army_formation[2],
        UnitType::Archer,
        player_translation,
        color,
        game_scene_id,
        4,
        commander,
    );

    commands.entity(commander).insert((
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

    Ok(())
}

fn spawn_unit_world(
    world: &mut World,
    player: Entity,
    commander: Entity,
    unit_type: UnitType,
    player_translation: Vec3,
    color: PlayerColor,
    game_scene_id: GameSceneId,
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
            game_scene_id,
        ))
        .id();

    let unit = Unit {
        swing_timer: Timer::from_seconds(4., TimerMode::Repeating),
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
        UnitType::Archer => 10.,
        UnitType::Bandit => todo!(),
        UnitType::Commander => todo!(),
    };
    let range = MeleeRange(range);

    for i in 1..=4 {
        world.spawn((
            Name::new(format!("Unit {}", i)),
            player_translation.with_layer(Layers::Flag),
            unit.clone(),
            health,
            speed,
            damage,
            range,
            owner,
            game_scene_id,
            FlagAssignment(flag_entity),
            UnitBehaviour::default(),
        ));
    }
    flag_entity
}
