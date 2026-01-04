use bevy::prelude::*;

use health::{Health, TakeDamage};
use shared::Owner;

use crate::{BuildStatus, Building};

pub(crate) struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (on_building_destroy, update_build_status).chain(),
        );
    }
}

fn on_building_destroy(
    mut query: Query<(
        Entity,
        &Health,
        &Building,
        &mut BuildStatus,
        &Owner,
        Option<&TargetedBy>,
    )>,
    mut commands: Commands,
) -> Result {
    for (entity, health, building, mut status, owner, maybe_targeted_by) in query.iter_mut() {
        if health.hitpoints <= 0. {
            *status = BuildStatus::Destroyed;

            commands
                .entity(entity)
                .remove::<Health>()
                .insert(Interactable {
                    kind: InteractionType::Building,
                    restricted_to: Some(owner.entity()?),
                });

            if let Some(targeted_by) = maybe_targeted_by {
                commands
                    .entity(entity)
                    .remove_related::<Target>(targeted_by);
            };

            if let BuildingType::MainBuilding { level: _ } = building.building_type {
                commands.server_trigger(ToClients {
                    mode: SendMode::Broadcast,
                    message: PlayerDefeated(owner.entity()?),
                });
            }
        }
    }
    Ok(())
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
