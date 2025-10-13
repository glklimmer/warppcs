use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{
    BoxCollider, ClientPlayerMap, DelayedDespawn, Owner, PlayerState, Vec3LayerExt,
    map::{
        Layers,
        buildings::{Building, BuildingType},
    },
    server::{
        buildings::recruiting::{FlagAssignment, FlagHolder},
        entities::{Unit, commander::ArmyFlagAssignments},
        game_scenes::GameSceneId,
        players::interaction::{Interactable, InteractionTriggeredEvent, InteractionType},
    },
};

pub struct PlayerPort;

impl Plugin for PlayerPort {
    fn build(&self, app: &mut App) {
        app.add_client_trigger::<ChannelPort>(Channel::Ordered)
            .add_observer(check_port_cooldown)
            .add_observer(spawn_player_portal)
            .add_systems(
                Update,
                (
                    channel_input
                        .before(ClientSet::Send)
                        .run_if(in_state(PlayerState::World)),
                    progress_cooldown.run_if(server_or_singleplayer),
                ),
            )
            .add_systems(
                FixedUpdate,
                port.run_if(on_event::<InteractionTriggeredEvent>),
            );
    }
}

#[derive(Event, Deserialize, Serialize)]
struct ChannelPort;

#[derive(Event)]
struct SpawnPortal;

#[derive(Component, Deref, DerefMut)]
struct PortCooldown(Timer);
impl Default for PortCooldown {
    fn default() -> Self {
        PortCooldown(Timer::from_seconds(30., TimerMode::Once))
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = portal_collider(),
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    Interactable{
        kind: InteractionType::Portal,
        restricted_to: None,
    },
)]
pub struct Portal;

fn portal_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(32., 32.),
        offset: Some(Vec2::new(0., 16.)),
    }
}

#[derive(Component, Deref)]
struct PortalDestination(Entity);

fn channel_input(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.pressed(KeyCode::KeyT) {
        commands.client_trigger(ChannelPort);
    }
}

fn check_port_cooldown(
    trigger: Trigger<FromClient<ChannelPort>>,
    mut commands: Commands,
    mut players: Query<Option<&mut PortCooldown>>,
    client_player_map: Res<ClientPlayerMap>,
) {
    let player = *client_player_map.get(&trigger.client_entity).unwrap();
    let Ok(maybe_cooldown) = players.get_mut(player) else {
        return;
    };

    let Some(mut cooldown) = maybe_cooldown else {
        commands
            .entity(player)
            .insert(PortCooldown::default())
            .trigger(SpawnPortal);
        return;
    };

    if (**cooldown).finished() {
        commands.entity(player).trigger(SpawnPortal);
        (**cooldown).reset();
    }
}

fn progress_cooldown(mut players: Query<&mut PortCooldown>, time: Res<Time>) {
    for mut cooldown in players.iter_mut() {
        (**cooldown).tick(time.delta());
    }
}

fn spawn_player_portal(
    trigger: Trigger<SpawnPortal>,
    mut commands: Commands,
    players: Query<(&Transform, &GameSceneId)>,
    main_buildings: Query<(&Building, &Owner, &Transform, &GameSceneId)>,
) {
    let player = trigger.target();
    let (player_transform, player_game_scene_id) = players.get(player).unwrap();

    let maybe_base = main_buildings.iter().find(|(building, owner, ..)| {
        match (building.building_type, owner.entity()) {
            (BuildingType::MainBuilding { .. }, Some(entity)) => entity.eq(&player),
            _ => false,
        }
    });
    let Some((.., base_transform, base_game_scene_id)) = maybe_base else {
        return;
    };

    let player_portal = commands.spawn_empty().id();
    let base_portal = commands.spawn_empty().id();

    commands.entity(player_portal).insert((
        Portal,
        player_transform.translation.with_layer(Layers::Building),
        *player_game_scene_id,
        PortalDestination(base_portal),
        DelayedDespawn(Timer::from_seconds(30., TimerMode::Once)),
    ));

    commands.entity(base_portal).insert((
        Portal,
        base_transform
            .translation
            .offset_x(-50.)
            .with_layer(Layers::Building),
        *base_game_scene_id,
        PortalDestination(player_portal),
        DelayedDespawn(Timer::from_seconds(30., TimerMode::Once)),
    ));
}

fn port(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    portal: Query<&PortalDestination>,
    destination: Query<(&Transform, &GameSceneId)>,
    flag_holders: Query<&FlagHolder>,
    commanders: Query<(&FlagAssignment, &ArmyFlagAssignments)>,
    units_on_flag: Query<(Entity, &FlagAssignment, &Unit)>,
) {
    for event in interactions.read() {
        let InteractionType::Portal = &event.interaction else {
            continue;
        };

        let player_entity = event.player;
        let target_portal = portal.get(event.interactable).unwrap();
        let (target_transform, target_game_scene_id) = destination.get(**target_portal).unwrap();
        let target_position = target_transform.translation;

        let flag_holder = flag_holders.get(player_entity);

        info!("Porting player...");

        let mut travel_entities = Vec::new();

        if let Ok(flag_holder) = flag_holder {
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

        commands.entity(player_entity).insert((
            Transform::from_xyz(
                target_position.x,
                target_position.y,
                Layers::Player.as_f32(),
            ),
            *target_game_scene_id,
        ));

        for entity in travel_entities {
            commands.entity(entity).insert((
                Transform::from_xyz(target_position.x, target_position.y, Layers::Unit.as_f32()),
                *target_game_scene_id,
            ));
        }
    }
}
