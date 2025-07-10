use bevy::prelude::*;

use crate::{Owner, map::buildings::BuildingType, networking::Inventory};

use super::BuildingChangeEnd;

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

pub fn enable_goldfarm(mut commands: Commands, mut events: EventReader<BuildingChangeEnd>) {
    for event in events.read() {
        let BuildingType::GoldFarm = event.building.building_type else {
            continue;
        };

        println!("Bought Gold Farm");

        commands
            .entity(event.0.building_entity)
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
