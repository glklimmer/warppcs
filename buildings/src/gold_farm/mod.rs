use bevy::prelude::*;

use health::Health;
use inventory::Inventory;
use lobby::PlayerColor;
use shared::{GameState, Owner, Vec3LayerExt, map::Layers};
use transport::Transport;

use crate::BuildingType;

use super::BuildingChangeEnd;

pub(crate) mod animation;

const GOLD_PER_TICK: u16 = 10;
const GOLD_TIMER: f32 = 2.;

pub(crate) struct GoldFarmPlugin;

impl Plugin for GoldFarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                gold_farm_output.run_if(in_state(GameState::GameSession)),
                enable_goldfarm.run_if(on_message::<BuildingChangeEnd>),
            ),
        );
    }
}

#[derive(Component)]
#[require(Inventory)]
pub struct GoldFarm {
    timer: Timer,
}

impl Default for GoldFarm {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(GOLD_TIMER, TimerMode::Repeating),
        }
    }
}

fn enable_goldfarm(mut commands: Commands, mut events: MessageReader<BuildingChangeEnd>) {
    for event in events.read() {
        let BuildingType::GoldFarm = event.building.building_type else {
            continue;
        };

        commands
            .entity(event.0.building_entity)
            .insert(GoldFarm::default());
    }
}

fn gold_farm_output(
    mut gold_farms_query: Query<(&mut GoldFarm, &mut Inventory)>,
    time: Res<Time>,
) -> Result {
    for (mut farm_timer, mut inventory) in &mut gold_farms_query {
        farm_timer.timer.tick(time.delta());

        if farm_timer.timer.just_finished() {
            inventory.gold += GOLD_PER_TICK;
            info!("gold farm inventory: {}", inventory.gold);
        }
    }
    Ok(())
}

// fn spawn_transport(mut commands: Commands) {
//     commands.spawn((
//         Transport,
//         transform.translation.with_layer(Layers::Unit),
//         Health { hitpoints: 100. },
//         PlayerColor,
//     ));
// }
