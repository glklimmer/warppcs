use bevy::{ecs::entity::MapEntities, prelude::*, sprite::Anchor};

use bevy_replicon::prelude::{Replicated, SendMode, ServerTriggerExt, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, Faction, Owner, flag_collider,
    map::{
        Layers,
        buildings::{Building, RecruitBuilding},
    },
    networking::{Inventory, UnitType},
    server::{
        ai::{
            UnitBehaviour,
            attack::{unit_health, unit_swing_timer},
        },
        entities::{Unit, commander::SlotsAssignments, health::Health},
        physics::attachment::AttachedTo,
        players::interaction::{
            Interactable, InteractableSound, InteractionTriggeredEvent, InteractionType,
        },
    },
};

#[derive(Component, Deserialize, Serialize)]
#[require(
    Replicated,
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
    BoxCollider(flag_collider),
    Transform( || Transform {translation: Vec3::new(0., 0., Layers::Flag.as_f32()) , scale: Vec3::splat(1./3.), ..default()}))]
pub struct Flag;

/// PlayerEntity is FlagHolder
#[derive(Component, Clone, Copy)]
pub struct FlagHolder(pub Entity);

#[derive(Component)]
pub struct FlagAssignment(pub Entity, pub Vec2);

#[derive(Event, Deserialize, Serialize)]
pub struct RecruitEvent {
    player: Entity,
    unit_type: UnitType,
}

pub fn recruit(
    mut commands: Commands,
    mut recruit: EventReader<RecruitEvent>,
    mut player_query: Query<(&Transform, &mut Inventory)>,
) {
    for event in recruit.read() {
        let player = event.player;
        let (player_transform, mut inventory) = player_query.get_mut(player).unwrap();
        let player_translation = player_transform.translation;
        let flag_translation = Vec3::new(
            player_translation.x,
            player_translation.y,
            Layers::Flag.as_f32(),
        );

        let cost = &event.unit_type.recruitment_cost();
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

        let (unit_type, unit_amount) = match event.unit_type {
            UnitType::Shieldwarrior => (UnitType::Shieldwarrior, 4),
            UnitType::Pikeman => (UnitType::Pikeman, 4),
            UnitType::Archer => (UnitType::Archer, 4),
            UnitType::Bandit => todo!(),
            UnitType::Commander => (UnitType::Commander, 1),
        };

        let unit = Unit {
            swing_timer: unit_swing_timer(&unit_type),
            unit_type,
        };
        let health = Health {
            hitpoints: unit_health(&unit_type),
        };

        for unit_number in 1..=unit_amount {
            let offset = Vec2::new(15. * (unit_number - 3) as f32 + 12., 0.);
            commands
                .spawn((
                    Transform::from_translation(flag_translation),
                    unit.clone(),
                    health.clone(),
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
    building: Query<&Building, With<RecruitBuilding>>,
) {
    for event in interactions.read() {
        let InteractionType::Recruit = &event.interaction else {
            continue;
        };
        let inventory = player.get(event.player).unwrap();
        let building = building.get(event.interactable).unwrap();
        let Building::Unit { weapon: unit_type } = *building else {
            continue;
        };

        let cost = unit_type.recruitment_cost();
        if !inventory.gold.ge(&cost.gold) {
            println!("Not enough gold for recruitment");
            continue;
        }

        recruit.send(RecruitEvent {
            player: event.player,
            unit_type,
        });
    }
}
