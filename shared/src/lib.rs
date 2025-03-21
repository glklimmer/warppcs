use bevy::{prelude::*, sprite::Anchor};
use bevy_replicon::prelude::*;

use bevy::{ecs::entity::MapEntities, math::bounding::Aabb2d};
use bevy_replicon_renet::RepliconRenetPlugins;
use map::Layers;
use player_attacks::PlayerAttacks;
use player_movement::PlayerMovement;
use serde::{Deserialize, Serialize};
use server::physics::movement::{Speed, Velocity};
use test_plugin::TestPlugin;

pub mod enum_map;
pub mod map;
pub mod networking;
pub mod player_attacks;
pub mod player_movement;
pub mod server;
pub mod steamworks;
pub mod test_plugin;

pub const GRAVITY_G: f32 = 9.81 * 100.;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RepliconPlugins.set(ServerPlugin {
                visibility_policy: VisibilityPolicy::All,
                ..Default::default()
            }),
            RepliconRenetPlugins,
            TestPlugin,
            PlayerMovement,
            PlayerAttacks,
        ))
        .replicate_group::<(PhysicalPlayer, Transform)>()
        .add_mapped_server_event::<AnimationChangeEvent>(ChannelKind::Ordered)
        .add_observer(spawn_clients)
        .add_systems(Startup, basic_map.run_if(server_or_singleplayer));
    }
}

fn basic_map(mut commands: Commands) {
    // commands.spawn(bundle)
}

#[derive(Clone, Copy, Debug, Deserialize, Event, Serialize)]
pub enum AnimationChange {
    Attack,
    Hit,
    Death,
}

#[derive(Clone, Copy, Debug, Deserialize, Event, Serialize)]
pub struct AnimationChangeEvent {
    pub entity: Entity,
    pub change: AnimationChange,
}

impl MapEntities for AnimationChangeEvent {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.entity = entity_mapper.map_entity(self.entity);
    }
}

fn spawn_clients(trigger: Trigger<ClientConnected>, mut commands: Commands) {
    info!("spawning player for `{:?}`", trigger.client_id);
    commands.spawn((
        PhysicalPlayer(trigger.client_id),
        Transform::from_xyz(50.0, 0.0, Layers::Player.as_f32()),
    ));
}

#[derive(Component, Deserialize, Serialize, Deref)]
#[require(
    Replicated,
    Transform(|| Transform::from_xyz(0., 0., Layers::Player.as_f32())),
    BoxCollider,
    Speed,
    Velocity,
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()})
)]
pub struct PhysicalPlayer(bevy_replicon::core::ClientId);

#[derive(Debug, Resource, Deref)]
pub struct LocalClientId(bevy_replicon::core::ClientId);

impl LocalClientId {
    pub const fn new(value: u64) -> Self {
        Self(ClientId::new(value))
    }
}

#[derive(Component, Copy, Clone, Default)]
pub struct BoxCollider {
    pub dimension: Vec2,
    pub offset: Option<Vec2>,
}

impl BoxCollider {
    pub fn half_size(&self) -> Vec2 {
        Vec2::new(self.dimension.x / 2., self.dimension.y / 2.)
    }

    pub fn at(&self, transform: &Transform) -> Aabb2d {
        Aabb2d::new(
            transform.translation.truncate() + self.offset.unwrap_or_default(),
            self.half_size(),
        )
    }

    pub fn at_pos(&self, position: Vec2) -> Aabb2d {
        Aabb2d::new(position + self.offset.unwrap_or_default(), self.half_size())
    }
}

pub fn player_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(50., 45.),
        offset: Some(Vec2::new(0., -23.)),
    }
}

pub fn unit_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(40., 35.),
        offset: Some(Vec2::new(0., -28.)),
    }
}

pub fn horse_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(40., 35.),
        offset: Some(Vec2::new(0., -28.)),
    }
}

pub fn projectile_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(20., 20.),
        offset: None,
    }
}

pub fn flag_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(45., 75.),
        offset: None,
    }
}

#[derive(Component)]
struct DelayedDespawn(Timer);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    GameSession,
}
