use serde::{Deserialize, Serialize};
use shared::enum_map::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Mappable)]
pub enum WallLevels {
    Basic,
    Wood,
    Tower,
}

impl WallLevels {
    pub(crate) fn next_level(&self) -> Option<WallLevels> {
        match self {
            WallLevels::Basic => Some(WallLevels::Wood),
            WallLevels::Wood => Some(WallLevels::Tower),
            WallLevels::Tower => None,
        }
    }
}
