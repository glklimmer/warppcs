use bevy::prelude::*;

use bevy::sprite::Anchor;
use health::Health;
use serde::{Deserialize, Serialize};

use crate::{BuildStatus, HealthIndicator, respawn::RespawnZone};

#[derive(Component, Serialize, Deserialize, Clone)]
#[require(
    Transform,
    RespawnZone,
    BuildStatus = BuildStatus::Built{indicator: HealthIndicator::Healthy},
    Sprite,
    Anchor::BOTTOM_CENTER,
    Health
)]
pub struct SiegeCamp {
    life: Timer,
}

impl Default for SiegeCamp {
    fn default() -> Self {
        Self {
            life: Timer::from_seconds(60. * 5., TimerMode::Once),
        }
    }
}

pub fn siege_camp_lifetime(
    mut commannds: Commands,
    mut query: Query<(Entity, &mut SiegeCamp)>,
    time: Res<Time>,
) {
    for (entity, mut siege_camp) in query.iter_mut() {
        siege_camp.life.tick(time.delta());

        if siege_camp.life.is_finished() {
            commannds.entity(entity).despawn();
        }
    }
}
