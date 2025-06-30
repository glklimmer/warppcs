use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;

use crate::{
    BoxCollider, Owner, Vec3LayerExt,
    map::{
        Layers,
        buildings::{Building, RespawnZone},
    },
    server::{
        ai::UnitBehaviour,
        players::items::{CalculatedStats, Effect, Item},
    },
};

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
        Option<&FlagUnits>,
    )>,
    mut respawn_zones: Query<(&mut RespawnZone, &Transform, &BoxCollider, &Owner)>,
    original_building: Query<(&Building, &ItemAssignment)>,
) {
    for (flag_entity, flag, flag_transform, flag_collider, flag_owner, maybe_flag_units) in
        flags.iter()
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

        let Ok((building, assignment)) = original_building.get(flag.original_building) else {
            // commander
            continue;
        };

        let items: Vec<Item> = assignment.items.clone().into_iter().flatten().collect();

        let num_alive = match maybe_flag_units {
            Some(units) => (**units).len() as i32,
            None => 0,
        };
        let max_allowed = items.calculated(Effect::UnitAmount) as i32;

        if num_alive < max_allowed {
            let (unit, health, speed, damage, range) =
                unit_stats(building.unit_type().unwrap(), &items, flag.color);

            commands.spawn((
                respawn_transform.translation.with_layer(Layers::Unit),
                unit.clone(),
                health,
                speed,
                damage,
                range,
                *flag_owner,
                FlagAssignment(flag_entity),
                UnitBehaviour::default(),
            ));
        }
    }
}
