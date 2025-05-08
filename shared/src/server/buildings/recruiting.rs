use bevy::{ecs::entity::MapEntities, prelude::*};

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{Replicated, SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, Faction, Owner, Vec3LayerExt, flag_collider,
    map::{
        Layers,
        buildings::{Building, RecruitBuilding},
    },
    networking::{Inventory, UnitType},
    server::{
        ai::UnitBehaviour,
        entities::{Damage, Range, Unit, commander::SlotsAssignments, health::Health},
        physics::{attachment::AttachedTo, movement::Speed},
        players::{
            interaction::{
                Interactable, InteractableSound, InteractionTriggeredEvent, InteractionType,
            },
            items::{CalculatedStats, Effect, Item, ItemType},
        },
    },
};

use super::item_assignment::ItemAssignment;

#[derive(Component, Deserialize, Serialize)]
#[require(
    Replicated,
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
    BoxCollider(flag_collider),
    Transform( || Transform {translation: Vec3::new(0., 0., Layers::Flag.as_f32()) , scale: Vec3::splat(1./3.), ..default()}))]
pub struct Flag;

/// PlayerEntity is FlagHolder
#[derive(Component, Clone, Copy, Deref, DerefMut, Deserialize, Serialize)]
pub struct FlagHolder(pub Entity);

impl MapEntities for FlagHolder {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        **self = entity_mapper.map_entity(**self);
    }
}

#[derive(Component, Deserialize, Serialize)]
pub struct FlagAssignment(pub Entity, pub Vec2);

impl MapEntities for FlagAssignment {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0);
    }
}

#[derive(Event, Deserialize, Serialize)]
pub struct RecruitEvent {
    player: Entity,
    unit_type: UnitType,
    items: Vec<Item>,
}

pub fn recruit(
    mut commands: Commands,
    mut recruit: EventReader<RecruitEvent>,
    mut player_query: Query<(&Transform, &mut Inventory)>,
) {
    for RecruitEvent {
        player,
        unit_type,
        items,
    } in recruit.read()
    {
        let player = *player;
        let unit_type = *unit_type;

        let (player_transform, mut inventory) = player_query.get_mut(player).unwrap();
        let player_translation = player_transform.translation;

        let cost = &unit_type.recruitment_cost();
        inventory.gold -= cost.gold;

        let owner = Owner(Faction::Player(player));
        // TODO: Refactor with Bevy 0.16 Parent API
        let flag_entity = commands
            .spawn((
                Flag,
                AttachedTo(player),
                Interactable {
                    kind: InteractionType::Flag,
                    restricted_to: Some(owner),
                },
                owner,
            ))
            .id();

        commands.entity(player).insert(FlagHolder(flag_entity));

        let unit_amount = if let UnitType::Commander = unit_type {
            1
        } else {
            items.calculated(Effect::UnitAmount) as i32
        };

        let time = items.calculated(Effect::AttackSpeed) / 2.;
        let unit = Unit {
            swing_timer: Timer::from_seconds(time, TimerMode::Repeating),
            unit_type,
        };

        let hitpoints = items.calculated(Effect::Health);
        let health = Health { hitpoints };

        let movement_speed = items.calculated(Effect::MovementSpeed);
        let speed = Speed(movement_speed);

        let damage = items.calculated(Effect::Damage);
        let damage = Damage(damage);

        let range = items.calculated(|item: &Item| {
            let ItemType::Weapon(weapon) = item.item_type else {
                return None;
            };
            Some(Effect::Range(weapon))
        });
        let range = Range(range);

        for unit_number in 1..=unit_amount {
            let offset = Vec2::new(15. * (unit_number - 3) as f32 + 12., 0.);
            commands
                .spawn((
                    player_translation.with_layer(Layers::Flag),
                    unit.clone(),
                    health,
                    speed,
                    damage,
                    range,
                    owner,
                    FlagAssignment(flag_entity, offset),
                    UnitBehaviour::FollowFlag(flag_entity, offset),
                ))
                .insert_if(
                    Interactable {
                        kind: InteractionType::CommanderInteraction,
                        restricted_to: Some(owner),
                    },
                    || unit_type.eq(&UnitType::Commander),
                )
                .insert_if(SlotsAssignments::default(), || {
                    unit_type.eq(&UnitType::Commander)
                });
        }

        commands.server_trigger(ToClients {
            mode: SendMode::Broadcast,
            event: InteractableSound {
                kind: InteractionType::Recruit,
                spatial_position: player_transform.translation,
            },
        });
    }
}

pub fn check_recruit(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut recruit: EventWriter<RecruitEvent>,
    player: Query<&Inventory>,
    building: Query<(&Building, &ItemAssignment), With<RecruitBuilding>>,
) {
    for event in interactions.read() {
        let InteractionType::Recruit = &event.interaction else {
            continue;
        };
        let inventory = player.get(event.player).unwrap();
        let (building, item_assignment) = building.get(event.interactable).unwrap();

        let unit_type = match *building {
            Building::MainBuilding { level } => Some(UnitType::Commander),
            Building::Unit { weapon: unit_type } => {
                let cost = unit_type.recruitment_cost();
                if !inventory.gold.ge(&cost.gold) {
                    println!("Not enough gold for recruitment");
                    continue;
                }
                Some(unit_type)
            }
            Building::Wall { level: _ } | Building::Tower | Building::GoldFarm => None,
        };

        let Some(unit_type) = unit_type else {
            continue;
        };

        recruit.send(RecruitEvent {
            player: event.player,
            unit_type,
            items: item_assignment
                .items
                .clone()
                .into_iter()
                .flatten()
                .collect(),
        });
    }
}
