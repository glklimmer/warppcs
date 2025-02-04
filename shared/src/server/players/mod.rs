use bevy::prelude::*;

use flag::{drop_flag, flag_interact, pick_flag, DropFlagEvent, PickFlagEvent};
use interaction::{InteractPlugin, InteractionTriggeredEvent};
use mount::mount;

use crate::{
    map::GameSceneId,
    networking::{PlayerCommand, ServerMessages},
};

use super::networking::{NetworkEvent, SendServerMessage, ServerLobby};

pub mod flag;
pub mod interaction;
pub mod mount;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InteractPlugin);

        app.add_event::<DropFlagEvent>();
        app.add_event::<PickFlagEvent>();

        app.add_systems(
            FixedUpdate,
            (
                attack.run_if(on_event::<NetworkEvent>),
                (mount, flag_interact).run_if(on_event::<InteractionTriggeredEvent>),
                drop_flag.run_if(on_event::<DropFlagEvent>),
                pick_flag.run_if(on_event::<PickFlagEvent>),
            ),
        );
    }
}

fn attack(
    mut network_events: EventReader<NetworkEvent>,
    mut sender: EventWriter<SendServerMessage>,
    scene_ids: Query<&GameSceneId>,
    lobby: Res<ServerLobby>,
) {
    for event in network_events.read() {
        let client_id = event.client_id;
        if let PlayerCommand::MeleeAttack = &event.message {
            if let Some(player_entity) = lobby.players.get(&client_id) {
                let game_scene_id = scene_ids.get(*player_entity).unwrap();
                sender.send(SendServerMessage {
                    message: ServerMessages::MeleeAttack {
                        entity: *player_entity,
                    },
                    game_scene_id: *game_scene_id,
                });
            }
        }
    }
}
