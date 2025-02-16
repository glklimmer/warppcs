use super::{GameScene, GameSceneId, GameSceneType};
use crate::{
    entities::{chest::chest, spawn_point::spawn_point},
    map::buildings::{gold_farm, main, marker, wall},
};

pub fn define_base_scene(id: GameSceneId) -> GameScene {
    GameScene {
        id,
        game_scene_type: GameSceneType::Base,
        slots: vec![
            main(0.),
            chest(200.),
            marker(400.),
            marker(-400.),
            marker(650.),
            wall(-1050.),
            wall(1050.),
            gold_farm(-800.),
            gold_farm(875.),
        ],
        left_portal: spawn_point(-1800.),
        right_portal: spawn_point(1800.),
    }
}
