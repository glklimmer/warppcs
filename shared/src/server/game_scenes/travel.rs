use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{
    FromClient, Replicated, SendMode, ServerTriggerExt, ToClients, server_or_singleplayer,
};
use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, ClientPlayerMap, ClientPlayerMapExt,
    map::Layers,
    server::{
        entities::commander::ArmyFlagAssignments,
        game_scenes::{
            GameSceneId,
            world::{GameScene, RevealMapNode},
        },
        physics::collider_trigger::ColliderTrigger,
        players::interaction::{ActiveInteraction, InteractionType},
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
        app.add_observer(start_travel)
            .add_systems(Update, travel_timer.run_if(server_or_singleplayer))
            .add_systems(
                FixedUpdate,
                (init_travel_dialog, end_travel).run_if(server_or_singleplayer),
            );
    }
}

#[derive(Component, Serialize, Deserialize)]
pub struct Traveling {
    pub source: GameScene,
    pub target: GameScene,
    time_left: Timer,
}

impl Traveling {
    fn between(source: GameScene, target: GameScene) -> Self {
        Self {
            source,
            target,
            time_left: Timer::from_seconds(5., TimerMode::Once),
        }
    }
}

#[derive(Component, Clone, Deref)]
pub struct TravelDestinations(Vec<Entity>);

impl TravelDestinations {
    pub fn new(destinations: Vec<Entity>) -> Self {
        Self(destinations)
    }
}

#[derive(Component, Clone, Deref)]
pub struct TravelDestinationOffset(f32);

impl TravelDestinationOffset {
    pub fn non_player() -> Self {
        Self(50.)
    }

    pub fn player() -> Self {
        Self(-50.)
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

#[derive(Event, Deserialize, Serialize)]
pub struct OpenTravelDialog;

#[derive(Event, Deserialize, Serialize, Deref)]
pub struct SelectTravelDestination(pub GameScene);

fn travel_timer(mut query: Query<&mut Traveling>, time: Res<Time>) {
    for mut traveling in &mut query {
        traveling.time_left.tick(time.delta());
    }
}

fn init_travel_dialog(
    mut traveling: EventReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    destinations: Query<&TravelDestinations>,
    game_scene: Query<&GameScene>,
    client_player_map: Res<ClientPlayerMap>,
) -> Result {
    for event in traveling.read() {
        let InteractionType::Travel = &event.interaction else {
            continue;
        };

        let player_entity = event.player;
        let destinations = destinations.get(event.interactable)?;
        let client = client_player_map.get_network_entity(&player_entity)?;

        for destination in &**destinations {
            let game_scene = game_scene.get(*destination)?;

            commands.server_trigger(ToClients {
                mode: SendMode::Direct(*client),
                event: RevealMapNode::to(*game_scene),
            });
        }

        commands.entity(player_entity).insert(ActiveInteraction {
            interactable: event.interactable,
        });
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*client),
            event: OpenTravelDialog,
        });
    }
    Ok(())
}

fn start_travel(
    trigger: Trigger<FromClient<SelectTravelDestination>>,
    flag_holders: Query<Option<&FlagHolder>>,
    commanders: Query<(&FlagAssignment, &ArmyFlagAssignments)>,
    units_on_flag: Query<(Entity, &FlagAssignment, &Unit)>,
    interaction: Query<&ActiveInteraction>,
    game_scenes: Query<&GameScene>,
    client_player_map: Res<ClientPlayerMap>,
    mut commands: Commands,
) -> Result {
    let selection = &**trigger.event();
    let player_entity = *client_player_map.get_player(&trigger.client_entity)?;

    let source = interaction.get(player_entity)?.interactable;
    let source = *game_scenes.get(source)?;

    let target = **selection;

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
                .filter(|(_, assignment, _)| slots_assignments.flags.contains(&Some(assignment.0)))
                .for_each(|(entity, assignment, _)| {
                    travel_entities.push(entity);
                    travel_entities.push(**assignment);
                });
        };
    };

    commands
        .entity(player_entity)
        .remove::<ActiveInteraction>()
        .insert(Traveling::between(source, target))
        .remove::<GameSceneId>();

    for entity in travel_entities {
        commands
            .entity(entity)
            .insert(Traveling::between(source, target))
            .remove::<GameSceneId>();
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

        let game_scene = travel.target;
        let target_entity = game_scene.entry_entity();

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

        let Ok(client) = client_player_map.get_network_entity(&entity) else {
            continue;
        };

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*client),
            event: RevealMapNode::to(travel.target),
        });
    }
    Ok(())
}
