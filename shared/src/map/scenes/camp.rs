use super::{GameScene, GameSceneId, GameSceneType};
use crate::entities::{chest::chest, spawn_point::spawn_point};

pub fn define_camp_scene(id: GameSceneId) -> GameScene {
    GameScene {
        id,
        game_scene_type: GameSceneType::Camp,
        slots: vec![chest(0.)],
        left_portal: spawn_point(-800.),
        right_portal: spawn_point(800.),
    }
}
