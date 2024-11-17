pub struct EconomyPlugin;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use serde::{Deserialize, Serialize};

use crate::{
    map::base::{BuildStatus, Upgradable, UpgradableBuilding},
    networking::{MultiplayerRoles, Owner, ServerChannel, ServerMessages},
    GameState,
};

use super::{buildings::UpgradableBuildingInteraction, networking::ServerLobby};

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
pub struct GoldAmount(pub u16);

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            enable_goldfarm.run_if(on_event::<UpgradableBuildingInteraction>()),
        );

        app.add_systems(
            FixedUpdate,
            (gold_farm_output).run_if(
                in_state(GameState::GameSession).and_then(in_state(MultiplayerRoles::Host)),
            ),
        );
    }
}

fn enable_goldfarm(
    mut commands: Commands,
    lobby: Res<ServerLobby>,
    mut enable: EventReader<UpgradableBuildingInteraction>,
    mut gold_player_query: Query<(Entity, &mut GoldAmount)>,
    mut gold_farms_query: Query<(
        &mut BuildStatus,
        //&Upgradable,
        &Owner,
    )>,

    mut server: ResMut<RenetServer>,
) {
    for event in enable.read() {
        if event.building_type.ne(&UpgradableBuilding::GoldFarm) {
            continue;
        }

        let client_id = event.client_id;
        let player_entity = lobby.players.get(&client_id).unwrap();
        let (_, mut gold_amount) = gold_player_query.get_mut(*player_entity).unwrap();

        if !gold_amount.0.gt(&50) {
            continue;
        }

        let (mut status, owner) = gold_farms_query.get_mut(event.entity).unwrap();

        if status.eq(&BuildStatus::None) {
            if owner.0.ne(&client_id) {
                continue;
            }

            gold_amount.0 -= 50;

            *status = BuildStatus::Built;
            // Add Upgradable::First Later
            commands
                .entity(event.entity)
                .insert(GoldFarmTimer::default());

            let message = ServerMessages::ChangeGoldAmount(gold_amount.clone());
            let message = bincode::serialize(&message).unwrap();
            server.send_message(client_id, ServerChannel::ServerMessages, message);
        }
    }
}

fn gold_farm_output(
    time: Res<Time>,
    lobby: Res<ServerLobby>,
    mut gold_farms_query: Query<(&BuildStatus, &mut GoldFarmTimer, &Owner)>,
    mut gold_player_query: Query<(Entity, &mut GoldAmount)>,
    mut server: ResMut<RenetServer>,
) {
    for (status, mut farm_timer, owner) in &mut gold_farms_query {
        if !status.eq(&BuildStatus::Built) {
            continue;
        }

        // Tick this specific farm's timer
        farm_timer.timer.tick(time.delta());

        // Check if this farm's timer is finished
        if farm_timer.timer.just_finished() {
            let player_entity = lobby.players.get(&owner.0).unwrap();
            let (_, mut gold_amount) = gold_player_query.get_mut(*player_entity).unwrap();
            gold_amount.0 += GOLD_PER_TICK;

            let message = ServerMessages::ChangeGoldAmount(gold_amount.clone());
            let message = bincode::serialize(&message).unwrap();
            server.send_message(owner.0, ServerChannel::ServerMessages, message);
        }
    }
}
