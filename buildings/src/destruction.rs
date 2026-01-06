use bevy::prelude::*;

use health::Health;

use crate::{BuildStatus, Building, HealthIndicator};

pub(crate) struct DestructionPlugin;

impl Plugin for DestructionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update_build_status);
    }
}

fn update_build_status(mut query: Query<(&Health, &mut BuildStatus, &Building), Changed<Health>>) {
    for (health, mut status, building) in query.iter_mut() {
        let percentage = health.hitpoints / building.health().hitpoints * 100.0;
        let percentage_i32 = percentage.clamp(0.0, 100.0) as i32;

        let severity = match percentage_i32 {
            90..=100 => HealthIndicator::Healthy,
            70..90 => HealthIndicator::Light,
            30..70 => HealthIndicator::Medium,
            _ => HealthIndicator::Heavy,
        };

        if let BuildStatus::Built { indicator } = *status
            && indicator != severity
        {
            *status = BuildStatus::Built {
                indicator: severity,
            };
        }
    }
}
