use bevy::prelude::*;
use bevy_replicon::prelude::*;

use bevy::ecs::entity::MapEntities;
use core::hash::{Hash, Hasher};
use map::Layers;
use serde::{Deserialize, Serialize};

pub mod enum_map;
pub mod map;
pub mod server;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RepliconPlugins.set(ServerPlugin {
            visibility_policy: VisibilityPolicy::Whitelist,
            ..Default::default()
        }))
        .replicate::<Owner>()
        .add_mapped_server_message::<AnimationChangeEvent>(Channel::Ordered);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Hitby {
    Arrow,
    Melee,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum AnimationChange {
    Idle,
    Attack,
    Hit(Hitby),
    Death,
    KnockOut,
    Mount,
    Unmount,
}

#[derive(Message, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct AnimationChangeEvent {
    pub entity: Entity,
    pub change: AnimationChange,
}

impl MapEntities for AnimationChangeEvent {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.entity = entity_mapper.get_mapped(self.entity);
    }
}

#[derive(Component, PartialEq, Eq, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct GameSceneId(usize);

impl GameSceneId {
    pub fn custom(id: usize) -> Self {
        Self(id)
    }

    pub fn lobby() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SceneType {
    Player { player: Entity, exit: Entity },
    Camp { left: Entity, right: Entity },
    Meadow { left: Entity, right: Entity },
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Copy)]
pub struct GameScene {
    pub id: GameSceneId,
    pub scene: SceneType,
    pub position: Vec2,
}

impl GameScene {
    pub fn entry_entity(&self) -> Entity {
        match self.scene {
            SceneType::Player { exit, .. } => exit,
            SceneType::Camp { left, .. } => left,
            SceneType::Meadow { left, .. } => left,
        }
    }
}

impl PartialEq for GameScene {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for GameScene {}

impl Hash for GameScene {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Component, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Owner {
    Player(Entity),
    Bandits,
}

impl Owner {
    pub fn entity(&self) -> Result<Entity, BevyError> {
        match self {
            Owner::Player(entity) => Ok(*entity),
            Owner::Bandits => Err(BevyError::from("Owner is not a player")),
        }
    }

    pub fn is_different_faction(&self, other: &Self) -> bool {
        match (self, other) {
            (Owner::Player { 0: id1 }, Owner::Player { 0: id2 }) => id1 != id2,
            (Owner::Player { .. }, Owner::Bandits) | (Owner::Bandits, Owner::Player { .. }) => true,
            (Owner::Bandits, Owner::Bandits) => false,
        }
    }

    pub fn is_same_faction(&self, other: &Self) -> bool {
        !self.is_different_faction(other)
    }
}

impl MapEntities for Owner {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        match self {
            Owner::Player(entity) => {
                *entity = entity_mapper.get_mapped(*entity);
            }
            Owner::Bandits => todo!(),
        }
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    GameSession,
    Paused,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerState {
    World,
    Interaction,
    Traveling,
    Respawn,
    Defeated,
}

pub trait Vec3LayerExt {
    fn offset_x(self, x: f32) -> Vec3;
    fn offset_y(self, y: f32) -> Vec3;
    fn offset_z(self, z: f32) -> Vec3;

    fn with_layer(self, layer: Layers) -> Transform;
}

impl Vec3LayerExt for Vec3 {
    fn offset_x(self, x: f32) -> Vec3 {
        self + Vec3::X * x
    }

    fn offset_y(self, y: f32) -> Vec3 {
        self + Vec3::Y * y
    }

    fn offset_z(self, z: f32) -> Vec3 {
        self + Vec3::Z * z
    }

    fn with_layer(self, layer: Layers) -> Transform {
        Transform::from_translation(Vec3::new(self.x, self.y, layer.as_f32()))
    }
}
