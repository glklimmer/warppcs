use serde::{Deserialize, Serialize};

pub const BRP_TRIGGER_SPAWN_UNIT: &str = "trigger/spawn_unit";

pub const BRP_TRIGGER_RANDOM_ITEM: &str = "trigger/random_items";

#[derive(Serialize, Deserialize)]
pub struct BrpSpawnUnit {
    pub unit: String,
    pub player: u8,
}
