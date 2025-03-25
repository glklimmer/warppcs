use bevy::{prelude::*, sprite::Anchor};

use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};

use crate::{
    flag_collider,
    map::{
        buildings::{Building, Cost},
        Layers,
    },
    networking::{Inventory, UnitType},
    server::{
        ai::{
            attack::{unit_health, unit_swing_timer},
            UnitBehaviour,
        },
        entities::{health::Health, Unit},
        physics::attachment::AttachedTo,
        players::interaction::{Interactable, InteractionTriggeredEvent, InteractionType},
    },
    BoxCollider, Faction, Owner,
};

#[derive(Component, Deserialize, Serialize)]
#[require(Replicated, Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}), BoxCollider(flag_collider), Transform)]
pub struct Flag;

/// PlayerEntity is FlagHolder
#[derive(Component)]
pub struct FlagHolder(pub Entity);

#[derive(Component)]
pub struct FlagAssignment(pub Entity, pub Vec2);

#[derive(Event)]
pub struct RecruitEvent {
    player: Entity,
    building_type: Building,
}

pub fn recruit(
    mut commands: Commands,
    mut recruit: EventReader<RecruitEvent>,
    mut player_query: Query<(&Transform, &mut Inventory)>,
) {
    for event in recruit.read() {
        let (player_transform, mut inventory) = player_query.get_mut(event.player).unwrap();
        let player_translation = player_transform.translation;
        let flag_translation = Vec3::new(
            player_translation.x,
            player_translation.y,
            Layers::Flag.as_f32(),
        );

        if let Some(cost) = recruitment_cost(&event.building_type) {
            inventory.gold -= cost.gold;
        } else {
            continue;
        }

        let owner = Owner(Faction::Player(event.player));
        // TODO: Refactor with Bevy 0.16 Parent API
        let flag_entity = commands
            .spawn((
                Flag,
                AttachedTo(event.player),
                Interactable {
                    kind: InteractionType::Flag,
                    restricted_to: Some(owner),
                },
                owner,
            ))
            .id();

        commands
            .entity(event.player)
            .insert(FlagHolder(flag_entity));

        let unit_type = match event.building_type {
            Building::Archer => UnitType::Archer,
            Building::Warrior => UnitType::Shieldwarrior,
            Building::Pikeman => UnitType::Pikeman,
            Building::Wall { level: _ }
            | Building::Tower
            | Building::GoldFarm
            | Building::MainBuilding { level: _ } => continue,
        };

        let unit = Unit {
            swing_timer: unit_swing_timer(&unit_type),
            unit_type,
        };
        let health = Health {
            hitpoints: unit_health(&unit_type),
        };

        for unit_number in 1..=4 {
            let offset = Vec2::new(15. * (unit_number - 3) as f32, 0.);
            commands.spawn((
                Transform::from_translation(flag_translation),
                unit.clone(),
                health.clone(),
                owner,
                FlagAssignment(flag_entity, offset),
                UnitBehaviour::FollowFlag(flag_entity, offset),
            ));
        }
    }
}

pub fn check_recruit(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut recruit: EventWriter<RecruitEvent>,
    player: Query<&Inventory>,
    building: Query<&Building>,
) {
    for event in interactions.read() {
        let InteractionType::Recruit = &event.interaction else {
            continue;
        };

        let inventory = player.get(event.player).unwrap();
        let building = building.get(event.interactable).unwrap();

        if let Some(cost) = recruitment_cost(building) {
            if !inventory.gold.ge(&cost.gold) {
                println!("Not enough gold for recruitment");
                continue;
            }
        } else {
            continue;
        }

        recruit.send(RecruitEvent {
            player: event.player,
            building_type: *building,
        });
    }
}

fn recruitment_cost(building_type: &Building) -> Option<Cost> {
    let gold = match building_type {
        Building::MainBuilding { level: _ }
        | Building::Wall { level: _ }
        | Building::Tower
        | Building::GoldFarm => return None,
        Building::Archer => 50,
        Building::Warrior => 50,
        Building::Pikeman => 50,
    };
    Some(Cost { gold })
}
