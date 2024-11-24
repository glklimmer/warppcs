pub struct EconomyPlugin;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use serde::{Deserialize, Serialize};

use crate::{
    map::base::Building,
    networking::{MultiplayerRoles, Owner, ServerChannel, ServerMessages},
    GameState,
};

use super::{buildings::BuildingConstruction, networking::ServerLobby};

const GOLD_PER_TICK: u16 = 10;
const GOLD_TIMER: f32 = 10.;

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

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Inventory {
    pub gold: u16,
}

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            enable_goldfarm.run_if(on_event::<BuildingConstruction>()),
        );

        app.add_systems(
            FixedUpdate,
            (gold_farm_output).run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
    }
}

fn enable_goldfarm(mut commands: Commands, mut builds: EventReader<BuildingConstruction>) {
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

fn gold_farm_output(
    time: Res<Time>,
    lobby: Res<ServerLobby>,
    mut gold_farms_query: Query<(&mut GoldFarmTimer, &Owner)>,
    mut inventory: Query<&mut Inventory>,
    mut server: ResMut<RenetServer>,
) {
    for (mut farm_timer, owner) in &mut gold_farms_query {
        farm_timer.timer.tick(time.delta());

        if farm_timer.timer.just_finished() {
            let player_entity = lobby.players.get(&owner.0).unwrap();
            let mut inventory = inventory.get_mut(*player_entity).unwrap();
            inventory.gold += GOLD_PER_TICK;

            let message = ServerMessages::SyncInventory(inventory.clone());
            let message = bincode::serialize(&message).unwrap();
            server.send_message(owner.0, ServerChannel::ServerMessages, message);
        }
    }
}
