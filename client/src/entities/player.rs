use bevy::prelude::*;

use crate::{
    animations::{king::KingAnimation, AnimationTrigger, FullAnimation},
    networking::{ClientPlayers, Connected, NetworkEvent, NetworkMapping, PlayerEntityMapping},
};
use shared::networking::{Mounted, ServerMessages};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (remove_player, mount_player)
                .run_if(on_event::<NetworkEvent>)
                .in_set(Connected),
        );
    }
}

fn remove_player(
    mut commands: Commands,
    mut network_mapping: ResMut<NetworkMapping>,
    mut network_events: EventReader<NetworkEvent>,
    mut lobby: ResMut<ClientPlayers>,
) {
    for event in network_events.read() {
        if let ServerMessages::PlayerDisconnected { id } = &event.message {
            println!("Player {} disconnected.", id);
            if let Some(PlayerEntityMapping {
                server_entity,
                client_entity,
            }) = lobby.players.remove(id)
            {
                commands.entity(client_entity).despawn();
                network_mapping.0.remove(&server_entity);
            }
        }
    }
}

fn mount_player(
    mut commands: Commands,
    mut network_events: EventReader<NetworkEvent>,
    mut animation_trigger: EventWriter<AnimationTrigger<KingAnimation>>,
    network_mapping: ResMut<NetworkMapping>,
) {
    for event in network_events.read() {
        let ServerMessages::Mount {
            entity: server_entity,
            mount_type,
        } = &event.message
        else {
            continue;
        };

        println!("starting mount animation");

        let player = network_mapping.0.get(server_entity).unwrap();
        commands.entity(*player).insert((
            Mounted {
                mount_type: *mount_type,
            },
            FullAnimation,
        ));

        animation_trigger.send(AnimationTrigger {
            entity: *player,
            state: KingAnimation::Mount,
        });
    }
}
