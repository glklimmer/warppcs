use bevy::{prelude::*, sprite::Anchor};
use bevy_replicon::prelude::{AppRuleExt, Channel, ClientEventAppExt, Replicated};
use serde::{Deserialize, Serialize};
use shared::{BoxCollider, PlayerColor, enum_map::*, map::Layers, networking::UnitType};

use crate::{commander::CommanderPlugin, flag::FlagPlugins, slot::ArmySlotPlugin};

pub mod commander;
pub mod flag;
pub mod slot;

pub struct ArmyPlugins;

impl Plugin for ArmyPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((FlagPlugins, ArmySlotPlugin, CommanderPlugin))
            .replicate::<ArmyFlagAssignments>()
            .replicate::<ArmyFormation>()
            .add_client_event::<ArmyPosition>(Channel::Ordered);
    }
}

#[derive(Event, Serialize, Deserialize, Copy, Clone, Mappable, PartialEq, Eq, Debug)]
enum ArmyPosition {
    Front,
    Middle,
    Back,
}

#[derive(Component, Serialize, Deserialize, Clone)]
struct ArmyFlagAssignments {
    #[entities]
    pub flags: EnumMap<ArmyPosition, Option<Entity>>,
}

impl Default for ArmyFlagAssignments {
    fn default() -> Self {
        Self {
            flags: EnumMap::new(|slot| match slot {
                ArmyPosition::Front => None,
                ArmyPosition::Middle => None,
                ArmyPosition::Back => None,
            }),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ArmyFormation {
    #[entities]
    pub positions: EnumMap<ArmyPosition, Entity>,
}
