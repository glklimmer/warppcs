use bevy::prelude::*;
use bevy_replicon::prelude::*;

use bevy::{
    ecs::entity::MapEntities, math::bounding::Aabb2d, platform::collections::HashMap,
    reflect::Reflect, sprite::Anchor,
};
use bevy_replicon::server::AuthorizedClient;
use core::hash::{Hash, Hasher};
use enum_map::*;
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
            ArmyFlagAssignments, ArmyPosition, CommanderAssignmentRequest,
            CommanderCampInteraction, CommanderInteraction,
        },
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

use crate::{
    player_port::{PlayerPort, Portal},
    server::{
        entities::{
            commander::{ArmyFormation, CommanderAssignmentReject, CommanderPickFlag},
            health::{Health, PlayerDefeated},
        },
        physics::army_slot::ArmySlot,
        players::{chest::ChestOpened, flag::FlagDestroyed},
    },
};

pub mod enum_map;
pub mod map;
pub mod networking;
pub mod player_attacks;
pub mod player_movement;
pub mod player_port;
pub mod server;

pub const GRAVITY_G: f32 = 9.81 * 33.;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RepliconPlugins.set(ServerPlugin {
                visibility_policy: VisibilityPolicy::Whitelist,
                ..Default::default()
            }),
            PlayerMovement,
            PlayerAttacks,
            PlayerPort,
            InteractPlugin,
        ))
        .init_resource::<ClientPlayerMap>()
        .replicate::<Moving>()
        .replicate::<Grounded>()
        .replicate::<BoxCollider>()
        .replicate::<Owner>()
        .replicate::<Mounted>()
        .replicate::<ItemAssignment>()
        .replicate::<Interactable>()
        .replicate::<AttachedTo>()
        .replicate::<ArmyFlagAssignments>()
        .replicate::<ArmyFormation>()
        .replicate::<FlagHolder>()
        .replicate::<FlagDestroyed>()
        .replicate::<ChestOpened>()
        .replicate_bundle::<(Player, Transform, Inventory)>()
        .replicate_bundle::<(RecruitBuilding, Transform)>()
        .replicate_bundle::<(Building, BuildStatus, Transform)>()
        .replicate_bundle::<(RespawnZone, Transform)>()
        .replicate_bundle::<(SiegeCamp, Transform)>()
        .replicate_bundle::<(Flag, Transform)>()
        .replicate_bundle::<(ProjectileType, Transform)>()
        .replicate_bundle::<(Unit, Transform)>()
        .replicate_bundle::<(Portal, Transform)>()
        .replicate_bundle::<(Mount, Transform)>()
        .replicate_bundle::<(Chest, Transform)>()
        .replicate_bundle::<(Item, Transform)>()
        .replicate_bundle::<(ArmySlot, Transform)>()
        .sync_related_entities::<FlagAssignment>()
        .add_client_event::<ArmyPosition>(Channel::Ordered)
        .add_client_event::<CommanderCampInteraction>(Channel::Ordered)
        .add_client_event::<AssignItem>(Channel::Ordered)
        .add_client_event::<StartBuild>(Channel::Ordered)
        .add_client_event::<CommanderAssignmentRequest>(Channel::Ordered)
        .add_client_event::<CommanderPickFlag>(Channel::Ordered)
        .add_server_event::<InteractableSound>(Channel::Ordered)
        .add_server_event::<CommanderAssignmentReject>(Channel::Ordered)
        .add_server_event::<CloseBuildingDialog>(Channel::Ordered)
        .add_mapped_server_event::<PlayerDefeated>(Channel::Ordered)
        .add_mapped_server_event::<CommanderInteraction>(Channel::Ordered)
        .add_mapped_server_event::<OpenBuildingDialog>(Channel::Ordered)
        .add_mapped_server_event::<SetLocalPlayer>(Channel::Ordered)
        .add_mapped_server_message::<AnimationChangeEvent>(Channel::Ordered)
        .add_observer(spawn_clients)
        .add_observer(update_visibility)
        .add_observer(hide_on_remove);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct PendingPlayers(HashMap<Entity, u64>);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Hitby {
    Arrow,
    Melee,
}
/// Key is NetworkEntity
/// Value is PlayerEntity
#[derive(Resource, DerefMut, Deref, Default, Reflect)]
pub struct ClientPlayerMap(HashMap<ClientId, Entity>);

impl ClientPlayerMap {
    pub fn get_network_entity(&self, value: &Entity) -> Result<&ClientId> {
        self.iter()
            .find_map(|(key, val)| if val == value { Some(key) } else { None })
            .ok_or("Network entity not found for player entity".into())
    }
}

pub trait ClientPlayerMapExt {
    fn get_player(&self, entity: &ClientId) -> Result<&Entity>;
}

impl ClientPlayerMapExt for ClientPlayerMap {
    fn get_player(&self, entity: &ClientId) -> Result<&Entity> {
        self.get(entity).ok_or("Player not found".into())
    }
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

    pub(crate) fn lobby() -> Self {
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

fn spawn_clients(
    trigger: On<Add, AuthorizedClient>,
    mut visibility: Query<&mut ClientVisibility>,
    mut client_player_map: ResMut<ClientPlayerMap>,
    mut commands: Commands,
    mut pending_players: ResMut<PendingPlayers>,
    disconnected_players: Query<(Entity, &Player), With<Disconnected>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let client_id = ClientId::Client(trigger.entity);
    let new_player_id = if let Some(id) = pending_players.remove(&trigger.entity) {
        id
    } else {
        trigger.entity.to_bits()
    };

    // Try to find a disconnected player with the same ID
    if let Some((player_entity, _)) = disconnected_players
        .iter()
        .find(|(_, player)| player.id == new_player_id)
    {
        // Player found, reconnect them
        commands.entity(player_entity).remove::<Disconnected>();
        client_player_map.insert(client_id, player_entity);

        for mut client_visibility in visibility.iter_mut() {
            client_visibility.set_visibility(player_entity, true);
        }

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(client_id),
            message: SetLocalPlayer(player_entity),
        });

        info!(
            "Player {:?} reconnected with id {}.",
            player_entity, new_player_id
        );

        // After this player is reconnected, there is one less disconnected player.
        // If there are no more disconnected players, unpause the game.
        if disconnected_players.iter().count() == 1 {
            game_state.set(GameState::GameSession);
            info!("All players reconnected, resuming game.");
        }
        return;
    }

    // No disconnected player found, spawn a new one
    let color = fastrand::choice(PlayerColor::all_variants()).unwrap();
    let player = commands.spawn_empty().id();
    commands.entity(player).insert((
        Player {
            id: new_player_id,
            color: *color,
        },
        Transform::from_xyz(250.0, 0.0, Layers::Player.as_f32()),
        GameSceneId::lobby(),
        Owner::Player(player),
        Health { hitpoints: 200. },
    ));

    client_player_map.insert(client_id, player);

    for mut client_visibility in visibility.iter_mut() {
        client_visibility.set_visibility(player, true);
    }

    commands.server_trigger(ToClients {
        mode: SendMode::Direct(client_id),
        message: SetLocalPlayer(player),
    });
    info!("New player {:?} spawned with id {}.", player, new_player_id);
}

fn update_visibility(
    trigger: On<Insert, GameSceneId>,
    client_player_map: Res<ClientPlayerMap>,
    mut visibility_query: Query<&mut ClientVisibility>,
    players_query: Query<(Entity, &GameSceneId), With<Player>>,
    others: Query<(Entity, &GameSceneId)>,
    player_check: Query<(), With<Player>>,
) -> Result {
    let entity = trigger.entity;
    let (_, new_entity_scene_id) = others.get(entity)?;

    if player_check.get(entity).is_ok() {
        let player_scenes: HashMap<Entity, GameSceneId> = players_query
            .iter()
            .map(|(entity, game_scene_id)| (entity, *game_scene_id))
            .collect();

        for (player_entity, _player_scene_id) in players_query.iter() {
            let client_entity = match client_player_map.get_network_entity(&player_entity)? {
                ClientId::Client(entity) => *entity,
                _ => continue,
            };

            if let Ok(mut visibility) = visibility_query.get_mut(client_entity) {
                if player_entity.eq(&entity) {
                    let player_scene_id = player_scenes
                        .get(&entity)
                        .ok_or("GameSceneId for player not found")?;
                    for (other_entity, other_scene_id) in &others {
                        visibility.set_visibility(other_entity, other_scene_id.eq(player_scene_id));
                    }
                } else {
                    let player_scene_id = player_scenes
                        .get(&player_entity)
                        .ok_or("GameSceneId for player not found")?;
                    visibility.set_visibility(entity, player_scene_id.eq(new_entity_scene_id));
                }
            }
        }
    } else {
        for (player_entity, player_scene_id) in players_query.iter() {
            let client_entity = match client_player_map.get_network_entity(&player_entity)? {
                ClientId::Client(entity) => *entity,
                _ => continue,
            };
            if let Ok(mut visibility) = visibility_query.get_mut(client_entity) {
                visibility.set_visibility(entity, player_scene_id.eq(new_entity_scene_id));
            }
        }
    }
    Ok(())
}

fn hide_on_remove(
    trigger: On<Remove, GameSceneId>,
    mut visibility_query: Query<&mut ClientVisibility>,
    players_query: Query<Entity, With<Player>>,
    client_player_map: Res<ClientPlayerMap>,
) {
    let entity = trigger.entity;
    for player_entity in players_query.iter() {
        if let Ok(client_id) = client_player_map.get_network_entity(&player_entity) {
            let client_entity = match client_id {
                ClientId::Client(e) => e,
                _ => continue,
            };
            if let Ok(mut visibility) = visibility_query.get_mut(*client_entity) {
                visibility.set_visibility(entity, player_entity == entity);
            }
        }
    }
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
    Transform = Transform::from_xyz(0., 0., Layers::Player.as_f32()),
    BoxCollider = player_collider(),
    Speed,
    Velocity,
    Sprite,
    Anchor::BOTTOM_CENTER,
    Inventory,
)]
pub struct Player {
    pub id: u64,
    pub color: PlayerColor,
}

#[derive(Component)]
pub struct Disconnected;

#[derive(Component)]
#[require(BoxCollider = player_collider())]
pub struct ControlledPlayer;

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

#[derive(Component)]
pub struct DelayedDespawn(Timer);

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
