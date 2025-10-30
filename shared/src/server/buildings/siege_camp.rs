use bevy::prelude::*;

use bevy::sprite::Anchor;
use serde::{Deserialize, Serialize};

use crate::{
    map::buildings::{BuildStatus, HealthIndicator, RespawnZone},
    server::entities::health::Health,
};

#[derive(Component, Serialize, Deserialize, Clone)]
#[require(
    Transform,
    RespawnZone,
    BuildStatus = BuildStatus::Built{indicator: HealthIndicator::Healthy},
    Sprite{anchor: Anchor::BottomCenter, ..default()},
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
) -> Result {
    for (entity, mut siege_camp) in query.iter_mut() {
        siege_camp.life.tick(time.delta());

        if siege_camp.life.finished() {
            commannds.entity(entity).despawn();
        }
    }
    Ok(())
}
