use aeronet::io::{Session, SessionEndpoint, connection::Disconnect};
use bevy::{
    ecs::query::QuerySingleError, input::common_conditions::input_just_pressed, prelude::*,
};

use bevy_egui::{EguiContexts, egui};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use super::enum_map::*;

use crate::{BoxCollider, horse_collider, map::buildings::Cost, server::players::items::Item};

pub const PROTOCOL_ID: u64 = 7;

#[derive(Resource)]
struct RepliconVisualizerState {
    pub show: bool,
}

pub struct NetworkRegistry;

impl Plugin for NetworkRegistry {
    fn build(&self, app: &mut App) {
        app.insert_resource(RepliconVisualizerState { show: false });
        app.add_client_event::<LobbyEvent>(Channel::Ordered);
        app.add_systems(
            Update,
            (
                replicon_visualizer,
                toggle_replicon_visualizer.run_if(input_just_pressed(KeyCode::KeyY)),
            ),
        );
    }
}

fn replicon_visualizer(
    mut commands: Commands,
    mut egui: EguiContexts,
    sessions: Query<(Entity, &Name, Option<&Session>), With<SessionEndpoint>>,
    replicon_client: Res<RepliconClient>,
    state: Res<RepliconVisualizerState>,
) {
    if !state.show {
        return;
    }
    let stats = replicon_client.stats();
    egui::Window::new("Replicon Visualizer").show(egui.ctx_mut(), |ui| {
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

#[derive(Debug, Deserialize, Event, Serialize)]
pub enum LobbyEvent {
    StartGame,
}

fn toggle_replicon_visualizer(mut state: ResMut<RepliconVisualizerState>) {
    state.show = !state.show;
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Mappable, PartialEq, Eq)]
pub enum UnitType {
    Shieldwarrior,
    Pikeman,
    Archer,
    Bandit,
    Commander,
}
impl UnitType {
    pub fn recruitment_cost(&self) -> Cost {
        let gold = match self {
            UnitType::Shieldwarrior => 50,
            UnitType::Pikeman => 50,
            UnitType::Archer => 50,
            UnitType::Bandit => todo!(),
            UnitType::Commander => 100,
        };
        Cost { gold }
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone, Copy)]
#[require(BoxCollider = horse_collider())]
pub enum MountType {
    Horse,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[require(Replicated)]
pub struct Inventory {
    pub gold: u16,
    pub items: Vec<Item>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            gold: 600,
            items: Vec::new(),
        }
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Mounted {
    pub mount_type: MountType,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum WorldDirection {
    #[default]
    Left,
    Right,
}

impl From<f32> for WorldDirection {
    fn from(value: f32) -> Self {
        match value > 0. {
            true => WorldDirection::Right,
            false => WorldDirection::Left,
        }
    }
}

impl From<WorldDirection> for f32 {
    fn from(value: WorldDirection) -> Self {
        match value {
            WorldDirection::Left => -1.,
            WorldDirection::Right => 1.,
        }
    }
}
