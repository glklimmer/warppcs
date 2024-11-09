pub struct EconomyPlugin;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use serde::{Deserialize, Serialize};

use crate::{
    map::base::{BuildStatus, Upgradable, UpgradableBuilding},
    networking::{Owner, ServerChannel, ServerMessages},
    GameState,
};

use super::{
    buildings::EnableUpgradableBuilding,
    networking::{ServerLobby, ServerPlayer},
};

#[derive(Resource)]
pub struct GoldTimer {
    pub timer: Timer,
}

impl Default for GoldTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Repeating),
        }
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct GoldAmount(pub u64);

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GoldTimer>();

        app.add_systems(
            FixedUpdate,
            enable_goldfarm.run_if(on_event::<EnableUpgradableBuilding>()),
        );

        app.add_systems(
            FixedUpdate,
            give_gold.run_if(in_state(GameState::GameSession)),
        );
    }
}

fn enable_goldfarm(
    mut commands: Commands,
    mut lobby: Res<ServerLobby>,
    mut enable: EventReader<EnableUpgradableBuilding>,
    mut gold_player_query: Query<(Entity, &mut GoldAmount)>,
    mut gold_farms_query: Query<(
        Entity,
        &mut BuildStatus,
        &UpgradableBuilding,
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

        for (entity, mut status, building, owner) in &mut gold_farms_query {
            if building.eq(&UpgradableBuilding::GoldFarm) {
                if owner.0.ne(&client_id) {
                    continue;
                }

                gold_amount.0 -= 50;
                *status = BuildStatus::Built;
                let message = ServerMessages::ChangeGoldAmount(gold_amount.clone());
                let message = bincode::serialize(&message).unwrap();
                server.send_message(client_id, ServerChannel::ServerMessages, message);

                println!("received");
                break;
            }

            // TODO check for upgrade level
            // if buidling.eq(&UpgradableBuilding::GoldFarm) {
            // if gold_amount.0.gt(&50) && status.ne(&BuildStatus::Built) {
            //     println!("Bought Farm");
            //     gold_amount.0 -= 50;
            //     *status = BuildStatus::Built;

            //     commands.entity(entity).insert(Upgradable::First);

            //     let message = ServerMessages::ChangeGoldAmount(gold_amount.clone());
            //     let message = bincode::serialize(&message).unwrap();
            //     server.send_message(client_id, ServerChannel::ServerMessages, message)
            // }
            // }
        }
    }
}

fn give_gold(
    time: Res<Time>,
    mut gold_timer: ResMut<GoldTimer>,
    lobby: Res<ServerLobby>,
    mut gold_player_query: Query<(Entity, &mut GoldAmount)>,
    mut gold_farms_query: Query<
        (&BuildStatus, &UpgradableBuilding, &Owner),
        With<UpgradableBuilding>,
    >,
    mut server: ResMut<RenetServer>,
) {
    // Tick the timer forward in time
    gold_timer.timer.tick(time.delta());

    // Only process gold giving when the timer has finished
    if !gold_timer.timer.just_finished() {
        return;
    }

    for (status, building, owner) in &mut gold_farms_query {
        if building.eq(&UpgradableBuilding::GoldFarm) && status.eq(&BuildStatus::Built) {
            let player_entity = lobby.players.get(&owner.0).unwrap();
            let (_, mut gold_amount) = gold_player_query.get_mut(*player_entity).unwrap();
            gold_amount.0 += 10;
            let message = ServerMessages::ChangeGoldAmount(gold_amount.clone());
            let message = bincode::serialize(&message).unwrap();
            server.send_message(owner.0, ServerChannel::ServerMessages, message)
        }
    }
}
