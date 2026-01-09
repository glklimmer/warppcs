use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use health::Health;
use physics::movement::{BoxCollider, Velocity};
use serde::{Deserialize, Serialize};
use shared::{Owner, enum_map::*};

use crate::{BuildStatus, Building, BuildingType};

pub(crate) mod animations;

pub(crate) struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, wall_collision);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Mappable)]
pub enum WallLevels {
    Basic,
    Wood,
    Tower,
}

impl WallLevels {
    pub(crate) fn next_level(&self) -> Option<WallLevels> {
        match self {
            WallLevels::Basic => Some(WallLevels::Wood),
            WallLevels::Wood => Some(WallLevels::Tower),
            WallLevels::Tower => None,
        }
    }
}

fn wall_collision(
    mut query: Query<(&mut Velocity, &Transform, &BoxCollider, &Owner)>,
    buildings: Query<(&Transform, &BoxCollider, &Owner, &Building, &BuildStatus), With<Health>>,
    time: Res<Time>,
) {
    for (mut velocity, transform, collider, owner) in query.iter_mut() {
        let future_position = transform.translation.truncate() + velocity.0 * time.delta_secs();
        let future_bounds = collider.at_pos(future_position);

        for (building_transform, building_collider, building_owner, building, building_status) in
            buildings.iter()
        {
            if building_owner.is_same_faction(owner) {
                continue;
            }

            let BuildStatus::Built { indicator: _ } = *building_status else {
                continue;
            };

            if let BuildingType::Wall { level: _ } = building.building_type {
                let building_bounds = building_collider.at(building_transform);

                if building_bounds.intersects(&future_bounds) {
                    velocity.0.x = 0.;
                    break;
                }
            }
        }
    }
}
