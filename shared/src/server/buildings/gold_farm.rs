use bevy::prelude::*;

use crate::{Owner, map::buildings::Building, networking::Inventory};

use super::BuildingConstruction;

const GOLD_PER_TICK: u16 = 10;
const GOLD_TIMER: f32 = 2.;

#[derive(Component)]
pub struct GoldFarmTimer {
    pub timer: Timer,
}

impl Default for GoldFarmTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(GOLD_TIMER, TimerMode::Repeating),
        }
    }
}

pub fn enable_goldfarm(mut commands: Commands, mut builds: EventReader<BuildingConstruction>) {
    for build in builds.read() {
        let Building::GoldFarm = build.0.building_type else {
            continue;
        };

        println!("Bought Gold Farm");

        commands
            .entity(build.0.entity)
            .insert(GoldFarmTimer::default());
    }
}

pub fn gold_farm_output(
    mut gold_farms_query: Query<(&mut GoldFarmTimer, &Owner)>,
    mut inventory_query: Query<&mut Inventory>,
    time: Res<Time>,
) {
    for (mut farm_timer, owner) in &mut gold_farms_query {
        farm_timer.timer.tick(time.delta());

        if farm_timer.timer.just_finished() {
            if let Ok(mut inventory) = inventory_query.get_mut(owner.entity().unwrap()) {
                inventory.gold += GOLD_PER_TICK;
            }
        }
    }
}
