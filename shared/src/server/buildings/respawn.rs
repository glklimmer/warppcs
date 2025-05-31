use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;

use crate::{
    BoxCollider, Owner,
    map::buildings::{Building, RecruitBuilding},
    server::{
        entities::health::Health,
        players::items::{CalculatedStats, Effect, Item},
    },
};

use super::{
    item_assignment::ItemAssignment,
    recruiting::{Flag, FlagUnits, spawn_unit},
};

pub fn respawn_units(
    mut commands: Commands,
    flags: Query<(Entity, &Flag, &FlagUnits, &Transform, &BoxCollider, &Owner)>,
    buildings: Query<(&RecruitBuilding, &Transform, &BoxCollider, &Owner)>,
    alive_units: Query<Entity, With<Health>>,
    original_building: Query<(&Building, &ItemAssignment)>,
) {
    for (flag_entity, flag, flag_units, flag_transform, flag_collider, flag_owner) in flags.iter() {
        let flag_bounds = flag_collider.at(flag_transform);
        let maybe_building = buildings
            .iter()
            .filter(|(.., transform, collider, _)| flag_bounds.intersects(&collider.at(transform)))
            .filter(|(recruit, ..)| recruit.respawn_timer_finished())
            .find(|(.., owner)| flag_owner.is_same_faction(owner));

        let Some((_, respawn_transform, ..)) = maybe_building else {
            continue;
        };

        let maybe_assignment = original_building.get(flag.original_building);
        let Ok((building, assignment)) = maybe_assignment else {
            // commander
            continue;
        };

        let items: Vec<Item> = assignment.items.clone().into_iter().flatten().collect();

        let flag_units = (**flag_units).clone();
        let alive_units = flag_units
            .iter()
            .filter(|unit| alive_units.get(**unit).is_ok())
            .count() as i32;

        if alive_units < (items.calculated(Effect::UnitAmount) as i32) {
            spawn_unit(
                commands.reborrow(),
                building.unit_type().unwrap(),
                &items,
                respawn_transform.translation,
                *flag_owner,
                flag_entity,
            );
        }
    }
}
