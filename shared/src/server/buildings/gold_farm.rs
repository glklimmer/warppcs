use bevy::prelude::*;

use crate::{map::buildings::Building, networking::Inventory, Faction, Owner};

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
        if build.0.building_type.ne(&Building::GoldFarm) {
            continue;
        }

        println!("Bought Gold Farm");

        commands
            .entity(build.0.entity)
            .insert(GoldFarmTimer::default());
    }
}

pub fn gold_farm_output(
    mut gold_farms_query: Query<(&mut GoldFarmTimer, &Owner)>,
    mut inventory: Query<&mut Inventory>,
    time: Res<Time>,
) {
    for (mut farm_timer, owner) in &mut gold_farms_query {
        farm_timer.timer.tick(time.delta());

        if farm_timer.timer.just_finished() {
            if let Faction::Player(entity) = **owner {
                let mut inventory = inventory.get_mut(entity).unwrap();
                inventory.gold += GOLD_PER_TICK;
            }
        }
    }
}
