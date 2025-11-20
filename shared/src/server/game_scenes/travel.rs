use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{
    Replicated, SendMode, ServerTriggerExt, ToClients, server_or_singleplayer,
};
use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, ClientPlayerMap,
    map::Layers,
    server::{
        entities::commander::ArmyFlagAssignments,
        game_scenes::{
            GameSceneId,
            world::{ExitType, GameScene, RevealMapNode},
        },
        physics::collider_trigger::ColliderTrigger,
        players::interaction::InteractionType,
    },
};

use super::super::{
    buildings::recruiting::{FlagAssignment, FlagHolder},
    entities::Unit,
    players::interaction::{Interactable, InteractionTriggeredEvent},
};

pub struct TravelPlugin;

impl Plugin for TravelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, travel_timer.run_if(server_or_singleplayer))
            .add_systems(
                FixedUpdate,
                (start_travel, end_travel).run_if(server_or_singleplayer),
            );
    }
}

#[derive(Component, Serialize, Deserialize)]
pub struct Traveling {
    pub source: (Entity, Option<GameScene>),
    pub target: (Entity, Option<GameScene>),
    time_left: Timer,
}

impl Traveling {
    fn player(
        source: Entity,
        source_game_scene: GameScene,
        target: Entity,
        target_game_scene: GameScene,
    ) -> Self {
        Self {
            source: (source, Some(source_game_scene)),
            target: (target, Some(target_game_scene)),
            time_left: Timer::from_seconds(5., TimerMode::Once),
        }
    }

    fn entity(source: Entity, target: Entity) -> Self {
        Self {
            source: (source, None),
            target: (target, None),
            time_left: Timer::from_seconds(5., TimerMode::Once),
        }
    }
}

#[derive(Component, Clone, Deref)]
pub struct TravelDestination(Entity);

impl TravelDestination {
    pub fn new(destination: Entity) -> Self {
        Self(destination)
    }
}

#[derive(Component, Clone, Deref, Default)]
pub struct TravelDestinationOffset(f32);

impl From<ExitType> for TravelDestinationOffset {
    fn from(value: ExitType) -> Self {
        let offset = match value {
            ExitType::Left => 50.,
            ExitType::Right => -50.,
        };
        Self(offset)
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = scene_end_collider(),
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    ColliderTrigger = ColliderTrigger::Travel
)]
pub struct SceneEnd;

fn scene_end_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(32., 32.),
        offset: Some(Vec2::new(0., 16.)),
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = road_collider(),
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    Interactable{
        kind: InteractionType::Travel,
        restricted_to: None,
    },
)]
pub struct Road;

fn road_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(32., 32.),
        offset: Some(Vec2::new(0., 16.)),
    }
}

fn travel_timer(mut query: Query<&mut Traveling>, time: Res<Time>) {
    for mut traveling in &mut query {
        traveling.time_left.tick(time.delta());
    }
}

fn start_travel(
    mut traveling: EventReader<InteractionTriggeredEvent>,
    flag_holders: Query<Option<&FlagHolder>>,
    commanders: Query<(&FlagAssignment, &ArmyFlagAssignments)>,
    units_on_flag: Query<(Entity, &FlagAssignment, &Unit)>,
    destination: Query<(Entity, &TravelDestination)>,
    game_scenes: Query<&GameScene>,
    mut commands: Commands,
) -> Result {
    for event in traveling.read() {
        let InteractionType::Travel = &event.interaction else {
            continue;
        };

        let player_entity = event.player;
        let (source, destination) = destination.get(event.interactable)?;
        let source_game_scene = game_scenes.get(source)?;
        let destination_game_scene = game_scenes.get(**destination)?;

        let flag_holder = flag_holders.get(player_entity)?;

        info!("Travel starting...");

        let mut travel_entities = Vec::new();

        if let Some(flag_holder) = flag_holder {
            units_on_flag
                .iter()
                .filter(|(_, assignment, _)| assignment.0 == flag_holder.0)
                .for_each(|(entity, _, _)| {
                    travel_entities.push(entity);
                    travel_entities.push(**flag_holder);
                });

            let commander = commanders
                .iter()
                .find(|(assignment, _)| assignment.0.eq(&flag_holder.0));

            if let Some((_, slots_assignments)) = commander {
                units_on_flag
                    .iter()
                    .filter(|(_, assignment, _)| {
                        slots_assignments.flags.contains(&Some(assignment.0))
                    })
                    .for_each(|(entity, assignment, _)| {
                        travel_entities.push(entity);
                        travel_entities.push(**assignment);
                    });
            };
        };

        commands
            .entity(player_entity)
            .insert(Traveling::player(
                source,
                *source_game_scene,
                **destination,
                *destination_game_scene,
            ))
            .remove::<GameSceneId>();

        for entity in travel_entities {
            commands
                .entity(entity)
                .insert(Traveling::entity(source, **destination))
                .remove::<GameSceneId>();
        }
    }
    Ok(())
}

fn end_travel(
    query: Query<(Entity, &Traveling)>,
    target: Query<(&Transform, &GameSceneId, Option<&TravelDestinationOffset>)>,
    client_player_map: Res<ClientPlayerMap>,
    mut commands: Commands,
) -> Result {
    for (entity, travel) in query.iter() {
        if !travel.time_left.finished() {
            continue;
        }

        let (target_entity, maybe_target_game_scene) = travel.target;

        let (target_transform, target_game_scene_id, maybe_offset) = target.get(target_entity)?;
        let target_position = target_transform.translation;

        info!("Travel finished to target position: {:?}", target_position);

        let travel_destination_offset = match maybe_offset {
            Some(offset) => **offset,
            None => 0.,
        };

        commands.entity(entity).remove::<Traveling>().insert((
            Transform::from_xyz(
                target_position.x + travel_destination_offset,
                target_position.y,
                Layers::Player.as_f32(),
            ),
            *target_game_scene_id,
        ));

        let Some(target_game_scene) = maybe_target_game_scene else {
            continue;
        };

        let Some(client) = (**client_player_map)
            .iter()
            .find_map(|(key, &val)| if val == entity { Some(key) } else { None })
        else {
            continue;
        };

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*client),
            event: RevealMapNode::to(target_game_scene),
        });
    }
    Ok(())
}
