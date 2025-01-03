use bevy::prelude::*;

use base::BaseSceneIndicator;
use camp::CampSceneIndicator;
use fight::FightSceneIndicator;
use serde::{Deserialize, Serialize};

pub mod base;
pub mod camp;
pub mod fight;

#[derive(Copy, Clone, Component, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SceneBuildingIndicator {
    Base(BaseSceneIndicator),
    Fight(FightSceneIndicator),
    Camp(CampSceneIndicator),
}
