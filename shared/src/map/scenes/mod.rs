use bevy::prelude::*;

use base::BaseSceneIndicator;
use fight::FightSceneIndicator;
use serde::{Deserialize, Serialize};

pub mod base;
pub mod fight;

#[derive(Copy, Clone, Component, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SceneBuildingIndicator {
    Base(BaseSceneIndicator),
    Fight(FightSceneIndicator),
}
