use serde::{Deserialize, Serialize};

use crate::map::buildings::{BuildingBundle, MainBuildingBundle};

#[derive(Clone, Copy)]
pub struct FightScene {
    pub left_main_building: MainBuildingBundle,
    pub left_archer_building: BuildingBundle,
    pub left_warrior_building: BuildingBundle,
    pub left_pikeman_building: BuildingBundle,
    pub left_left_wall: BuildingBundle,
    pub left_right_wall: BuildingBundle,
    pub left_gold_farm: BuildingBundle,

    pub right_main_building: MainBuildingBundle,
    pub right_archer_building: BuildingBundle,
    pub right_warrior_building: BuildingBundle,
    pub right_pikeman_building: BuildingBundle,
    pub right_left_wall: BuildingBundle,
    pub right_right_wall: BuildingBundle,
    pub right_gold_farm: BuildingBundle,
}

impl FightScene {
    pub fn new() -> Self {
        FightScene {
            right_main_building: MainBuildingBundle::new(1500.),
            right_archer_building: BuildingBundle::archer(1900.),
            right_warrior_building: BuildingBundle::warrior(1100.),
            right_pikeman_building: BuildingBundle::pikeman(2150.),
            right_left_wall: BuildingBundle::wall(700.),
            right_right_wall: BuildingBundle::wall(2550.),
            right_gold_farm: BuildingBundle::gold_farm(2800.),

            left_main_building: MainBuildingBundle::new(-1500.),
            left_archer_building: BuildingBundle::archer(-1100.),
            left_warrior_building: BuildingBundle::warrior(-1900.),
            left_pikeman_building: BuildingBundle::pikeman(-850.),
            left_left_wall: BuildingBundle::wall(-2300.),
            left_right_wall: BuildingBundle::wall(-450.),
            left_gold_farm: BuildingBundle::gold_farm(-2800.),
        }
    }
}

impl Default for FightScene {
    fn default() -> Self {
        Self::new()
    }
}
