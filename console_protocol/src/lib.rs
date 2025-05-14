use serde::{Deserialize, Serialize};

pub const BRP_SPAWN_UNIT: &str = "player/spawn_unit";

#[derive(Serialize, Deserialize)]
pub struct BrpSpawnUnit {
    pub player: u8,
    pub unit: String,
}

pub const BRP_SPAWN_RANDOM_ITEM: &str = "player/spawn_random_items";

#[derive(Serialize, Deserialize)]
pub struct BrpSpawnItems {
    pub player: u8,
}
