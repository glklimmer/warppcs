use bevy::{platform::collections::HashMap, prelude::*};
use bevy_replicon::{
    RepliconPlugins,
    prelude::{
        AppRuleExt, Channel, ClientTriggerAppExt, ConnectedClient, Replicated, SendMode,
        ServerEventAppExt, ServerTriggerAppExt, ServerTriggerExt, SyncRelatedAppExt, ToClients,
    },
    server::{ServerPlugin, VisibilityPolicy},
};
use enum_map::*;

use bevy::{ecs::entity::MapEntities, math::bounding::Aabb2d, sprite::Anchor};
use map::{
    Layers,
    buildings::{BuildStatus, Building, RecruitBuilding, RespawnZone},
};
use networking::{Inventory, Mounted};
use player_attacks::PlayerAttacks;
use player_movement::PlayerMovement;
use serde::{Deserialize, Serialize};
use server::{
    buildings::{
        item_assignment::{
            AssignItem, CloseBuildingDialog, ItemAssignment, OpenBuildingDialog, StartBuild,
        },
        recruiting::{Flag, FlagAssignment, FlagHolder},
        siege_camp::SiegeCamp,
    },
    entities::{
        Unit,
        commander::{
            ArmyFlagAssignments, CommanderAssignmentRequest, CommanderCampInteraction,
            CommanderFormation, CommanderInteraction,
        },
    },
    game_scenes::{
        map::{GameScene, LoadMap},
        travel::{Portal, Traveling},
    },
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

use crate::server::entities::commander::{
    ArmyFormation, CommanderAssignmentReject, CommanderPickFlag,
};

pub mod enum_map;
pub mod map;
pub mod networking;
pub mod player_attacks;
pub mod player_movement;
pub mod server;

pub const GRAVITY_G: f32 = 9.81 * 33.;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RepliconPlugins.set(ServerPlugin {
                visibility_policy: VisibilityPolicy::All,
                ..Default::default()
            }),
            PlayerMovement,
            PlayerAttacks,
            InteractPlugin,
        ))
        .init_resource::<ClientPlayerMap>()
        .replicate::<Moving>()
        .replicate::<Grounded>()
        .replicate::<BoxCollider>()
        .replicate::<Owner>()
        .replicate::<Mounted>()
        .replicate::<ItemAssignment>()
        .replicate::<Traveling>()
        .replicate::<GameScene>()
        .replicate::<Interactable>()
        .replicate::<AttachedTo>()
        .replicate::<ArmyFlagAssignments>()
        .replicate::<ArmyFormation>()
        .replicate::<FlagHolder>()
        .replicate_group::<(Player, Transform, Inventory)>()
        .replicate_group::<(RecruitBuilding, Transform)>()
        .replicate_group::<(Building, BuildStatus, Transform)>()
        .replicate_group::<(RespawnZone, Transform)>()
        .replicate_group::<(SiegeCamp, Transform)>()
        .replicate_group::<(Flag, Transform)>()
        .replicate_group::<(ProjectileType, Transform)>()
        .replicate_group::<(Unit, Transform)>()
        .replicate_group::<(Portal, Transform)>()
        .replicate_group::<(Mount, Transform)>()
        .replicate_group::<(Chest, Transform)>()
        .replicate_group::<(Item, Transform)>()
        .sync_related_entities::<FlagAssignment>()
        .add_client_trigger::<CommanderFormation>(Channel::Ordered)
        .add_client_trigger::<CommanderCampInteraction>(Channel::Ordered)
        .add_client_trigger::<AssignItem>(Channel::Ordered)
        .add_client_trigger::<StartBuild>(Channel::Ordered)
        .add_client_trigger::<CommanderAssignmentRequest>(Channel::Unordered)
        .add_client_trigger::<CommanderPickFlag>(Channel::Unordered)
        .add_server_trigger::<InteractableSound>(Channel::Ordered)
        .add_server_trigger::<CommanderAssignmentReject>(Channel::Unordered)
        .add_server_trigger::<CloseBuildingDialog>(Channel::Ordered)
        .add_server_trigger::<LoadMap>(Channel::Ordered)
        .add_mapped_server_trigger::<CommanderInteraction>(Channel::Ordered)
        .add_mapped_server_trigger::<OpenBuildingDialog>(Channel::Ordered)
        .add_mapped_server_trigger::<SetLocalPlayer>(Channel::Ordered)
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
#[derive(Resource, DerefMut, Deref, Default, Reflect)]
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
    Unmount,
}

#[derive(Event, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct AnimationChangeEvent {
    pub entity: Entity,
    pub change: AnimationChange,
}

impl MapEntities for AnimationChangeEvent {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.entity = entity_mapper.get_mapped(self.entity);
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
        self.entity = entity_mapper.get_mapped(self.entity);
    }
}

fn spawn_clients(
    trigger: Trigger<OnAdd, ConnectedClient>,
    mut commands: Commands,
    mut client_player_map: ResMut<ClientPlayerMap>,
) {
    let player = commands
        .entity(trigger.target())
        .insert((
            Player {
                color: *fastrand::choice(PlayerColor::all_variants()).unwrap(),
            },
            Transform::from_xyz(50.0, 0.0, Layers::Player.as_f32()),
        ))
        .id();

    commands.server_trigger(ToClients {
        mode: SendMode::Direct(trigger.target()),
        event: SetLocalPlayer(player),
    });

    client_player_map.insert(trigger.target(), player);
}

#[derive(Event, Clone, Copy, Debug, Deserialize, Serialize, Deref, DerefMut)]
pub struct SetLocalPlayer(Entity);

impl MapEntities for SetLocalPlayer {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.get_mapped(self.0);
    }
}

#[derive(Component, Deserialize, Serialize)]
#[require(
    Replicated,
    Transform = (Transform::from_xyz(0., 0., Layers::Player.as_f32())),
    BoxCollider = player_collider(),
    Speed,
    Velocity,
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    Inventory,
)]
pub struct Player {
    pub color: PlayerColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Mappable, Serialize, Deserialize)]
pub enum PlayerColor {
    #[default]
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    Orange,
    Cyan,
    Magenta,
    Pink,
    Brown,
    Teal,
    Gray,
}

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

#[derive(Debug, Component, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Owner {
    Player(Entity),
    Bandits,
}

impl Owner {
    pub fn entity(&self) -> Option<Entity> {
        match self {
            Owner::Player(entity) => Some(*entity),
            Owner::Bandits => None,
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

#[derive(Component)]
struct DelayedDespawn(Timer);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    GameSession,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerState {
    World,
    Interaction,
    Traveling,
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
