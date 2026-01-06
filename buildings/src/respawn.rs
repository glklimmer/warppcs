use bevy::prelude::*;

use army::{
    ArmyFlagAssignments,
    flag::{Flag, FlagAssignment, FlagUnits},
};
use bevy::math::bounding::IntersectsVolume;
use bevy_replicon::prelude::Replicated;
use inventory::Inventory;
use items::{CalculatedStats, Effect, Item};
use physics::movement::BoxCollider;
use serde::{Deserialize, Serialize};
use shared::{GameSceneId, Owner, Vec3LayerExt, map::Layers};

use crate::{Building, recruiting::unit_stats};

use super::item_assignment::ItemAssignment;

const RESPAWN_COST_GOLD: u16 = 20;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = BoxCollider{
        dimension: Vec2::new(28., 26.),
        offset: Some(Vec2::new(0., 13.)),
    },
)]
pub struct RespawnZone {
    respawn_timer: Timer,
}

impl Default for RespawnZone {
    fn default() -> Self {
        Self {
            respawn_timer: Timer::from_seconds(2., TimerMode::Once),
        }
    }
}

impl RespawnZone {
    pub fn respawn_timer_finished(&self) -> bool {
        self.respawn_timer.is_finished()
    }

    pub fn respawn_timer_reset(&mut self) {
        self.respawn_timer.reset();
    }
}

pub fn respawn_timer(mut recruit_buildings: Query<&mut RespawnZone>, time: Res<Time>) {
    for mut recruit_building in &mut recruit_buildings.iter_mut() {
        recruit_building.respawn_timer.tick(time.delta());
    }
}

#[allow(clippy::type_complexity)]
pub fn respawn_units(
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
    mut commands: Commands,
) -> Result {
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

        let Ok(player) = flag_owner.entity() else {
            continue;
        };
        let mut inventory = inventory_query.get_mut(player)?;

        match original_building.get(flag.original_building) {
            Ok((building, assignment)) => {
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
                    let (flag, maybe_flag_units) = flag_query.get(*flag_entity)?;
                    let (building, assignment) = original_building.get(flag.original_building)?;

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
    Ok(())
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
        ));
    }
}
