use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::AppRuleExt;
use health::Health;
use serde::{Deserialize, Serialize};

use crate::{
    BuildStatus, HealthIndicator, respawn::RespawnZone,
    siege_camp::animations::SiegeCampAnimationPlugin,
};

pub(crate) mod animations;

pub(crate) struct SiegeCampPlugin;

impl Plugin for SiegeCampPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SiegeCampAnimationPlugin)
            .replicate_bundle::<(SiegeCamp, Transform)>()
            .add_systems(FixedUpdate, (siege_camp_lifetime,));
    }
}

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

fn siege_camp_lifetime(
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
