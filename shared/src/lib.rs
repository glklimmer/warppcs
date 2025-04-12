use bevy::{prelude::*, utils::HashMap};
use bevy_replicon::prelude::*;
use enum_map::*;

use bevy::{ecs::entity::MapEntities, math::bounding::Aabb2d, sprite::Anchor};
use bevy_replicon_renet::RepliconRenetPlugins;
use map::{
    Layers,
    buildings::{BuildStatus, Building},
};
use networking::{Inventory, Mounted};
use player_attacks::PlayerAttacks;
use player_movement::PlayerMovement;
use serde::{Deserialize, Serialize};
use server::{
    buildings::recruiting::Flag,
    entities::Unit,
    game_scenes::Portal,
    physics::{
        attachment::AttachedTo,
        movement::{Grounded, Moving, Speed, Velocity},
        projectile::ProjectileType,
    },
    players::{
        chest::Chest,
        interaction::{InteractPlugin, Interactable, InteractableSound},
        items::Item,
        mount::Mount,
    },
};

pub mod enum_map;
pub mod map;
pub mod networking;
pub mod player_attacks;
pub mod player_movement;
pub mod server;
pub mod steamworks;

pub const GRAVITY_G: f32 = 9.81 * 33.;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RepliconPlugins.set(ServerPlugin {
                visibility_policy: VisibilityPolicy::All,
                ..Default::default()
            }),
            RepliconRenetPlugins,
            PlayerMovement,
            PlayerAttacks,
            InteractPlugin,
        ))
        .init_resource::<ClientPlayerMap>()
        .replicate::<Moving>()
        .replicate::<Grounded>()
        .replicate_mapped::<AttachedTo>()
        .replicate::<BoxCollider>()
        .replicate::<Mounted>()
        .replicate_mapped::<Interactable>()
        .replicate_group::<(Player, Transform, Inventory)>()
        .replicate_group::<(Building, BuildStatus, Transform)>()
        .replicate_group::<(Flag, Transform)>()
        .replicate_group::<(ProjectileType, Transform)>()
        .replicate_group::<(Unit, Transform)>()
        .replicate_group::<(Portal, Transform)>()
        .replicate_group::<(Mount, Transform)>()
        .replicate_group::<(Chest, Transform)>()
        .replicate_group::<(Item, Transform)>()
        .add_server_trigger::<InteractableSound>(Channel::Ordered)
        .add_mapped_server_event::<SetLocalPlayer>(Channel::Ordered)
        .add_mapped_server_event::<AnimationChangeEvent>(Channel::Ordered)
        .add_mapped_server_event::<ChestAnimationEvent>(Channel::Ordered)
        .add_observer(spawn_clients);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Hitby {
    Arrow,
    Melee,
}
/// Key is NetworkEntity
/// Value is PlayerEntity
#[derive(Resource, DerefMut, Deref, Default)]
pub struct ClientPlayerMap(HashMap<Entity, Entity>);

impl ClientPlayerMap {
    pub fn get_network_entity(&self, value: &Entity) -> Option<&Entity> {
        self.iter()
            .find_map(|(key, val)| if val == value { Some(key) } else { None })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum AnimationChange {
    Attack,
    Hit(Hitby),
    Death,
    Mount,
}

#[derive(Event, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct AnimationChangeEvent {
    pub entity: Entity,
    pub change: AnimationChange,
}

impl MapEntities for AnimationChangeEvent {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.entity = entity_mapper.map_entity(self.entity);
    }
}

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Deserialize, Serialize)]
pub enum ChestAnimation {
    Open,
    Close,
}

#[derive(Event, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ChestAnimationEvent {
    pub entity: Entity,
    pub animation: ChestAnimation,
}

impl MapEntities for ChestAnimationEvent {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.entity = entity_mapper.map_entity(self.entity);
    }
}

fn spawn_clients(
    trigger: Trigger<OnAdd, ConnectedClient>,
    mut commands: Commands,
    mut set_local_player: EventWriter<ToClients<SetLocalPlayer>>,
    mut client_player_map: ResMut<ClientPlayerMap>,
) {
    info!("spawning player for `{:?}`", trigger.entity());

    let player = commands
        .entity(trigger.entity())
        .insert((
            Player,
            Transform::from_xyz(50.0, 0.0, Layers::Player.as_f32()),
        ))
        .id();

    set_local_player.send(ToClients {
        mode: SendMode::Direct(trigger.entity()),
        event: SetLocalPlayer(player),
    });

    client_player_map.insert(trigger.entity(), player);
}

#[derive(Event, Clone, Copy, Debug, Deserialize, Serialize, Deref, DerefMut)]
pub struct SetLocalPlayer(Entity);

impl MapEntities for SetLocalPlayer {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        **self = entity_mapper.map_entity(**self);
    }
}

#[derive(Component, Deserialize, Serialize)]
#[require(
    Replicated,
    Transform(|| Transform::from_xyz(0., 0., Layers::Player.as_f32())),
    BoxCollider(player_collider),
    Speed,
    Velocity,
    Sprite(|| Sprite{anchor: Anchor::BottomCenter, ..default()}),
    Inventory
)]
pub struct Player;

#[derive(Component, Copy, Clone, Default, Deserialize, Serialize)]
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
        dimension: Vec2::new(16., 16.),
        offset: Some(Vec2::new(0., 8.)),
    }
}

pub fn unit_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 16.),
        offset: Some(Vec2::new(0., 8.)),
    }
}

pub fn horse_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(16., 16.),
        offset: Some(Vec2::new(0., 8.)),
    }
}

pub fn projectile_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(14., 3.),
        offset: Some(Vec2::new(1.0, 0.)),
    }
}

pub fn flag_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(15., 20.),
        offset: Some(Vec2::new(0., 10.)),
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Faction {
    Player(Entity),
    Bandits,
}

#[derive(Debug, Component, Eq, PartialEq, Serialize, Deserialize, Copy, Clone, Deref)]
pub struct Owner(Faction);

impl Owner {
    pub fn is_different_faction(&self, other: &Self) -> bool {
        match (self.0, other.0) {
            // Two players - compare client IDs
            (Faction::Player { 0: id1 }, Faction::Player { 0: id2 }) => id1 != id2,
            // Different enum variants means different factions
            (Faction::Player { .. }, Faction::Bandits)
            | (Faction::Bandits, Faction::Player { .. }) => true,
            // Both bandits are the same faction
            (Faction::Bandits, Faction::Bandits) => false,
        }
    }

    pub fn is_same_faction(&self, other: &Self) -> bool {
        !self.is_different_faction(other)
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

trait Vec3LayerExt {
    fn offset_x(self, x: f32) -> Vec3;
    fn offset_z(self, z: f32) -> Vec3;

    fn with_layer(self, layer: Layers) -> Transform;
}

impl Vec3LayerExt for Vec3 {
    fn offset_x(self, x: f32) -> Vec3 {
        self + Vec3::X * x
    }

    fn offset_z(self, z: f32) -> Vec3 {
        self + Vec3::Z * z
    }

    fn with_layer(self, layer: Layers) -> Transform {
        Transform::from_translation(self.offset_z(layer.as_f32()))
    }
}
