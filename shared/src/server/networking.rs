use aeronet::io::{Session, SessionEndpoint, connection::Disconnect};
use bevy::{ecs::query::QuerySingleError, prelude::*};
use bevy_egui::{EguiContexts, egui};
use bevy_replicon::prelude::{RepliconClient, RepliconClientStatus};

use super::{
    ai::AIPlugin, buildings::BuildingsPlugins, console::ConsolePlugin,
    create_server::CreateServerPlugin, entities::EntityPlugin, game_scenes::GameScenesPlugin,
    physics::PhysicsPlugin, players::PlayerPlugin,
};
use crate::networking::NetworkRegistry;

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CreateServerPlugin,
            NetworkRegistry,
            AIPlugin,
            PhysicsPlugin,
            GameScenesPlugin,
            BuildingsPlugins,
            PlayerPlugin,
            EntityPlugin,
            ConsolePlugin,
        ));
        app.add_systems(Update, global_ui);
    }
}

fn global_ui(
    mut commands: Commands,
    mut egui: EguiContexts,
    sessions: Query<(Entity, &Name, Option<&Session>), With<SessionEndpoint>>,
    replicon_client: Res<RepliconClient>,
) {
    let stats = replicon_client.stats();
    egui::Window::new("Session Log").show(egui.ctx_mut(), |ui| {
        ui.label("Replicon reports:");
        ui.horizontal(|ui| {
            ui.label(match replicon_client.status() {
                RepliconClientStatus::Disconnected => "Disconnected",
                RepliconClientStatus::Connecting => "Connecting",
                RepliconClientStatus::Connected => "Connected",
            });
            ui.separator();

            ui.label(format!("Round-time trip {:.0}ms", stats.rtt * 1000.0));
            ui.separator();

            ui.label(format!(
                "Packet loss Loss {:.1}%",
                stats.packet_loss * 100.0
            ));
            ui.separator();

            ui.label(format!("Bytes received {:.0}bps", stats.received_bps));
            ui.separator();

            ui.label(format!("Bytes sent {:.0}bps", stats.sent_bps));
        });
        match sessions.single() {
            Ok((session, name, connected)) => {
                if connected.is_some() {
                    ui.label(format!("{name} connected"));
                } else {
                    ui.label(format!("{name} connecting"));
                }

                if ui.button("Disconnect").clicked() {
                    commands.trigger_targets(Disconnect::new("pressed disconnect button"), session);
                }
            }
            Err(QuerySingleError::NoEntities(_)) => {
                ui.label("No sessions active");
            }
            Err(QuerySingleError::MultipleEntities(_)) => {
                ui.label("Multiple sessions active");
            }
        }

        ui.separator();
    });
}
