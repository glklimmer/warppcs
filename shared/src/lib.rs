use bevy::prelude::*;
use bevy_replicon::prelude::*;

use bevy::color::palettes::css::BLUE;
use bevy::math::bounding::Aabb2d;
use bevy_replicon_renet::RepliconRenetPlugins;
use player_movement::PlayerMovement;
use serde::{Deserialize, Serialize};
use server::physics::movement::{Speed, Velocity};
use test_plugin::TestPlugin;

pub mod enum_map;
pub mod map;
pub mod networking;
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
        ))
        .replicate_group::<(PhysicalPlayer, Transform)>()
        .add_observer(spawn_clients)
        .add_systems(Startup, basic_map.run_if(server_or_singleplayer))
        .add_systems(Update, draw_boxes);
    }
}

fn basic_map(mut commands: Commands) {
    // commands.spawn(bundle)
}

const PIXEL_SCALE: Vec3 = Vec3::new(3., 3., 1.);

fn spawn_clients(trigger: Trigger<ClientConnected>, mut commands: Commands) {
    info!("spawning player for `{:?}`", trigger.client_id);
    commands.spawn((
        PhysicalPlayer(trigger.client_id),
        Transform::from_xyz(50.0, 0.0, 0.0).with_scale(PIXEL_SCALE),
    ));
}

fn draw_boxes(mut gizmos: Gizmos, players: Query<&Transform, With<PhysicalPlayer>>) {
    for transform in &players {
        gizmos.rect(
            Vec3::new(transform.translation.x, transform.translation.y, 0.0),
            Vec2::ONE * 50.0,
            BLUE,
        );
    }
}

#[derive(Component, Deserialize, Serialize, Deref)]
#[require(Replicated, Transform(|| Transform::from_scale(PIXEL_SCALE)), BoxCollider, Speed, Velocity, Sprite)]
pub struct PhysicalPlayer(bevy_replicon::core::ClientId);

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
