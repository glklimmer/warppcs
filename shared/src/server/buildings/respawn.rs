use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;

use crate::{
    BoxCollider, Owner, Vec3LayerExt,
    map::{
        Layers,
        buildings::{Building, RespawnZone},
    },
    networking::Inventory,
    server::{
        ai::UnitBehaviour,
        entities::commander::ArmyFlagAssignments,
        game_scenes::GameSceneId,
        players::items::{CalculatedStats, Effect, Item},
    },
};

const RESPAWN_COST_GOLD: u16 = 20;

use super::{
    item_assignment::ItemAssignment,
    recruiting::{Flag, FlagAssignment, FlagUnits, unit_stats},
};

#[allow(clippy::type_complexity)]
pub fn respawn_units(
    mut commands: Commands,
    flags: Query<(
        Entity,
        &Flag,
        &Transform,
        &BoxCollider,
        &Owner,
        &GameSceneId,
        Option<&FlagUnits>,
    )>,
    mut respawn_zones: Query<(&mut RespawnZone, &Transform, &BoxCollider, &Owner)>,
    original_building: Query<(&Building, &ItemAssignment)>,
    commander: Query<&ArmyFlagAssignments>,
    flag_query: Query<(&Flag, Option<&FlagUnits>)>,
    mut inventory_query: Query<&mut Inventory>,
) {
    for (
        flag_entity,
        flag,
        flag_transform,
        flag_collider,
        flag_owner,
        flag_game_scene_id,
        maybe_flag_units,
    ) in flags.iter()
    {
        let flag_bounds = flag_collider.at(flag_transform);

        let matching_building =
            respawn_zones
                .iter_mut()
                .find(|(recruit, transform, collider, owner)| {
                    if !recruit.respawn_timer_finished() {
                        return false;
                    }
                    if !flag_owner.is_same_faction(owner) {
                        return false;
                    }
                    let building_bounds = collider.at(transform);
                    flag_bounds.intersects(&building_bounds)
                });

        let Some((mut recruit_component, respawn_transform, ..)) = matching_building else {
            continue;
        };

        recruit_component.respawn_timer_reset();

        match original_building.get(flag.original_building) {
            Ok((building, assignment)) => {
                let player = match flag_owner.entity() {
                    Some(entity) => entity,
                    None => continue,
                };
                let Ok(mut inventory) = inventory_query.get_mut(player) else {
                    continue;
                };

                respawn_for_flag(
                    commands.reborrow(),
                    flag_entity,
                    flag,
                    flag_owner,
                    maybe_flag_units,
                    respawn_transform,
                    building,
                    assignment,
                    &mut inventory,
                    flag_game_scene_id,
                );
            }

            Err(_) => {
                // Commander
                let Some(flag_units) = maybe_flag_units else {
                    continue;
                };
                let Some(entity) = flag_units.first() else {
                    continue;
                };
                let Ok(assignment) = commander.get(*entity) else {
                    continue;
                };
                for flag_entity in assignment.flags.iter().flatten() {
                    let (flag, maybe_flag_units) = flag_query.get(*flag_entity).unwrap();
                    let (building, assignment) =
                        original_building.get(flag.original_building).unwrap();

                    let player = match flag_owner.entity() {
                        Some(entity) => entity,
                        None => continue,
                    };
                    let Ok(mut inventory) = inventory_query.get_mut(player) else {
                        continue;
                    };

                    respawn_for_flag(
                        commands.reborrow(),
                        *flag_entity,
                        flag,
                        flag_owner,
                        maybe_flag_units,
                        respawn_transform,
                        building,
                        assignment,
                        &mut inventory,
                        flag_game_scene_id,
                    );
                }
            }
        };
    }
}

fn respawn_for_flag(
    mut commands: Commands,
    flag_entity: Entity,
    flag: &Flag,
    flag_owner: &Owner,
    maybe_flag_units: Option<&FlagUnits>,
    respawn_transform: &Transform,
    building: &Building,
    assignment: &ItemAssignment,
    inventory: &mut Inventory,
    game_scene_id: &GameSceneId,
) {
    let items: Vec<Item> = assignment.items.clone().into_iter().flatten().collect();

    let num_alive = match maybe_flag_units {
        Some(units) => (**units).len() as i32,
        None => 0,
    };
    let max_allowed = items.calculated(Effect::UnitAmount) as i32;

    if num_alive < max_allowed {
        if inventory.gold < RESPAWN_COST_GOLD {
            return;
        }
        inventory.gold -= RESPAWN_COST_GOLD;

        let (unit, health, speed, damage, melee_range, projectile_range, sight) =
            unit_stats(building.unit_type().unwrap(), &items, flag.color);

        commands.spawn((
            respawn_transform.translation.with_layer(Layers::Unit),
            unit.clone(),
            health,
            speed,
            damage,
            melee_range,
            projectile_range,
            sight,
            *flag_owner,
            *game_scene_id,
            FlagAssignment(flag_entity),
            UnitBehaviour::default(),
        ));
    }
}
