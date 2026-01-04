use serde::{Deserialize, Serialize};
use shared::enum_map::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Mappable)]
pub enum MainBuildingLevels {
    Tent,
    Hall,
    Castle,
}

impl MainBuildingLevels {
    pub(crate) fn next_level(&self) -> Option<MainBuildingLevels> {
        match self {
            MainBuildingLevels::Tent => Some(MainBuildingLevels::Hall),
            MainBuildingLevels::Hall => Some(MainBuildingLevels::Castle),
            MainBuildingLevels::Castle => None,
        }
    }
}
