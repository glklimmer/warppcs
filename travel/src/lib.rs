use bevy::prelude::*;

use animations::ui::map_icon::{MapIconSpriteSheet, MapIcons};
use bevy::{input::common_conditions::input_just_pressed, sprite::Anchor};
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientTriggerAppExt, ClientTriggerExt, FromClient, Replicated, SendMode,
    ServerTriggerAppExt, ServerTriggerExt, ToClients, server_or_singleplayer,
};
use highlight::{
    Highlightable,
    utils::{add_highlight_on, remove_highlight_on},
};
use serde::{Deserialize, Serialize};
use shared::{
    BoxCollider, ClientPlayerMap, ClientPlayerMapExt, ControlledPlayer, GameScene, GameSceneId,
    GameState, PlayerState, SceneType,
    map::Layers,
    server::{
        buildings::recruiting::{FlagAssignment, FlagHolder},
        entities::{Unit, commander::ArmyFlagAssignments},
        players::interaction::{
            ActiveInteraction, Interactable, InteractionTriggeredEvent, InteractionType,
        },
    },
};

pub struct TravelPlugin;

impl Plugin for TravelPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<Traveling>()
            .replicate_group::<(Road, Transform)>()
            .replicate_group::<(SceneEnd, Transform)>()
            .add_client_trigger::<SelectTravelDestination>(Channel::Ordered)
            .add_server_trigger::<AddMysteryMapIcon>(Channel::Ordered)
            .add_server_trigger::<RevealMapIcon>(Channel::Ordered)
            .add_server_trigger::<OpenTravelDialog>(Channel::Ordered)
            .add_server_trigger::<InitPlayerMapNode>(Channel::Ordered)
            .insert_state(MapState::View)
            .add_observer(init_map)
            .add_observer(enter_travel_state)
            .add_observer(leave_travel_state)
            .add_observer(add_map_icons)
            .add_observer(reveal_map_icons)
            .add_observer(open_travel_dialog)
            .add_observer(start_travel)
            .add_systems(
                OnEnter(PlayerState::Traveling),
                (show_map, spawn_travel_dashline),
            )
            .add_systems(OnExit(PlayerState::Traveling), hide_map)
            .add_systems(
                Update,
                (
                    sync_ui_to_camera,
                    toggle_map
                        .run_if(input_just_pressed(KeyCode::KeyM))
                        .run_if(not(in_state(PlayerState::Traveling)))
                        .run_if(in_state(GameState::GameSession)),
                    animate_dashes,
                ),
            )
            .add_systems(Update, travel_timer.run_if(server_or_singleplayer))
            .add_systems(
                FixedUpdate,
                (init_travel_dialog, end_travel).run_if(server_or_singleplayer),
            );
    }
}

#[derive(Event, Deref, Serialize, Deserialize)]
pub struct InitPlayerMapNode(GameScene);

impl InitPlayerMapNode {
    pub fn new(game_scene: GameScene) -> Self {
        Self(game_scene)
    }
}

#[derive(Event, Deserialize, Serialize)]
pub struct OpenTravelDialog {
    pub current_scene: GameScene,
}

#[derive(Event, Deserialize, Serialize, Deref)]
pub struct SelectTravelDestination(pub GameScene);

#[derive(Event, Deref, Serialize, Deserialize)]
pub struct AddMysteryMapIcon(GameScene);

#[derive(Event, Deref, Serialize, Deserialize)]
pub struct RevealMapIcon(GameScene);

#[derive(Component, Serialize, Deserialize)]
pub struct Traveling {
    source: GameScene,
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

#[derive(Component, Deref)]
struct MapNode(GameScene);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum MapState {
    View,
    Selection { current_scene: GameScene },
}

#[derive(Component)]
pub struct DashLine {
    pub a: Vec2,
    pub b: Vec2,
    pub dash_len: f32,
    pub gap: f32,
    pub thickness: f32,
    pub color: Color,

    pub cp1: Vec2,
    pub cp2: Vec2,
    pub total_dashes: usize,
    pub spawned: usize,
    pub timer: Timer,
}

#[derive(Component, Default)]
struct UIElement;

#[derive(Component)]
#[require(UIElement)]
struct Map;

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
        let Ok(destinations) = destinations.get(event.interactable) else {
            continue;
        };
        let Ok(client) = client_player_map.get_network_entity(&player_entity) else {
            continue;
        };

        for destination in &**destinations {
            let Ok(game_scene) = game_scene.get(*destination) else {
                continue;
            };

            commands.server_trigger(ToClients {
                mode: SendMode::Direct(*client),
                event: AddMysteryMapIcon(*game_scene),
            });
        }

        let Ok(current_scene) = game_scene.get(event.interactable) else {
            continue;
        };

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*client),
            event: OpenTravelDialog {
                current_scene: *current_scene,
            },
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
            event: RevealMapIcon(travel.target),
        });
    }
    Ok(())
}

fn init_map(
    trigger: Trigger<InitPlayerMapNode>,
    assets: Res<AssetServer>,
    map_icons: Res<MapIconSpriteSheet>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) -> Result {
    let player_scene = **trigger.event();
    let map_texture = assets.load::<Image>("sprites/ui/map.png");

    next_game_state.set(GameState::GameSession);

    commands
        .spawn((
            Map,
            Visibility::Hidden,
            Sprite::from_image(map_texture),
            Transform::from_scale(Vec3::splat(1.0 / 3.0)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    MapNode(player_scene),
                    Visibility::Inherited,
                    Sprite::from_atlas_image(
                        map_icons.sprite_sheet.texture.clone(),
                        map_icons.sprite_sheet.texture_atlas(MapIcons::Player),
                    ),
                    Highlightable::default(),
                    Pickable::default(),
                    Transform::from_xyz(player_scene.position.x, player_scene.position.y, 2.0),
                ))
                .observe(add_highlight_on::<Pointer<Over>>)
                .observe(remove_highlight_on::<Pointer<Out>>)
                .observe(destination_selected);
        });
    Ok(())
}

fn open_travel_dialog(
    trigger: Trigger<OpenTravelDialog>,
    mut map: Query<&mut Visibility, With<Map>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
    mut next_map_state: ResMut<NextState<MapState>>,
) -> Result {
    let mut map = map.single_mut()?;
    *map = Visibility::Visible;

    next_player_state.set(PlayerState::Interaction);

    next_map_state.set(MapState::Selection {
        current_scene: trigger.current_scene,
    });

    Ok(())
}

fn destination_selected(
    trigger: Trigger<Pointer<Released>>,
    mut commands: Commands,
    map_state: ResMut<State<MapState>>,
    mut next_map_state: ResMut<NextState<MapState>>,
    map_node: Query<&MapNode>,
) -> Result {
    let MapState::Selection { current_scene } = map_state.get() else {
        return Ok(());
    };

    let entity = trigger.target();
    let game_scene = **map_node.get(entity)?;

    if game_scene.eq(current_scene) {
        return Ok(());
    }

    next_map_state.set(MapState::View);

    commands.client_trigger(SelectTravelDestination(game_scene));
    Ok(())
}

fn add_map_icons(
    trigger: Trigger<AddMysteryMapIcon>,
    map: Query<Entity, With<Map>>,
    map_icons: Res<MapIconSpriteSheet>,
    query: Query<(Entity, &MapNode)>,
    mut commands: Commands,
) -> Result {
    let map = map.single()?;

    let update_map_icon = trigger.event();
    let game_scene = **update_map_icon;

    let None = query.iter().find(|(_, node)| node.eq(&game_scene)) else {
        return Ok(());
    };

    let icon = MapIcons::Mystery;

    commands
        .spawn((
            ChildOf(map),
            MapNode(game_scene),
            Visibility::Inherited,
            Sprite::from_atlas_image(
                map_icons.sprite_sheet.texture.clone(),
                map_icons.sprite_sheet.texture_atlas(icon),
            ),
            Highlightable::default(),
            Pickable::default(),
            Transform::from_xyz(game_scene.position.x, game_scene.position.y, 2.0),
        ))
        .observe(add_highlight_on::<Pointer<Over>>)
        .observe(remove_highlight_on::<Pointer<Out>>)
        .observe(destination_selected);
    Ok(())
}

fn reveal_map_icons(
    trigger: Trigger<RevealMapIcon>,
    query: Query<(Entity, &MapNode)>,
    map_icons: Res<MapIconSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let update_map_icon = trigger.event();
    let game_scene = **update_map_icon;

    info!("revealing: {:?}", game_scene.scene);

    let (entity, _) = query
        .iter()
        .find(|(_, node)| node.eq(&game_scene))
        .ok_or("GameScene not added yet.")?;

    let icon = match game_scene.scene {
        SceneType::Player { .. } => MapIcons::Player,
        SceneType::Camp { .. } => MapIcons::Bandit,
        SceneType::Meadow { .. } => MapIcons::Bandit,
    };

    commands.entity(entity).insert(Sprite::from_atlas_image(
        map_icons.sprite_sheet.texture.clone(),
        map_icons.sprite_sheet.texture_atlas(icon),
    ));
    Ok(())
}

impl DashLine {
    pub fn new(
        a: Vec2,
        b: Vec2,
        dash_len: f32,
        gap: f32,
        thickness: f32,
        wiggle: f32,
        color: Color,
        total_time: f32,
    ) -> Self {
        let dir = (b - a).normalize();
        let perp = Vec2::new(-dir.y, dir.x);
        let t1 = fastrand::f32() * 0.5 + 0.1;
        let t2 = fastrand::f32() * 0.5 + 0.4;
        let cp1 = a.lerp(b, t1) + perp * (fastrand::f32() * 2.0 - 1.0) * wiggle;
        let cp2 = a.lerp(b, t2) + perp * (fastrand::f32() * 2.0 - 1.0) * wiggle;

        let straight_len = a.distance(b);
        let total_dashes = (straight_len / (dash_len + gap)).floor() as usize;

        let per_dash_secs = total_time / (total_dashes as f32).max(1.0);

        DashLine {
            a,
            b,
            dash_len,
            gap,
            thickness,
            color,
            cp1,
            cp2,
            total_dashes,
            spawned: 0,
            timer: Timer::from_seconds(per_dash_secs, TimerMode::Repeating),
        }
    }
}

fn bezier(a: Vec2, c1: Vec2, c2: Vec2, b: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;
    a * uuu + c1 * 3.0 * uu * t + c2 * 3.0 * u * tt + b * ttt
}

fn bezier_tangent(a: Vec2, c1: Vec2, c2: Vec2, b: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    3.0 * u * u * (c1 - a) + 6.0 * u * t * (c2 - c1) + 3.0 * t * t * (b - c2)
}

fn animate_dashes(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DashLine)>,
    time: Res<Time>,
    map: Query<Entity, With<Map>>,
) -> Result {
    for (entity, mut line) in query.iter_mut() {
        line.timer.tick(time.delta());

        if line.timer.finished() && line.spawned < line.total_dashes {
            let t = (line.spawned as f32 + 0.5) / line.total_dashes as f32;
            let pos = bezier(line.a, line.cp1, line.cp2, line.b, t);
            let tan = bezier_tangent(line.a, line.cp1, line.cp2, line.b, t);
            let angle = tan.to_angle();

            let dash = commands
                .spawn((
                    Sprite {
                        color: line.color,
                        custom_size: Some(Vec2::new(line.dash_len, line.thickness)),
                        ..Default::default()
                    },
                    Transform {
                        translation: pos.extend(1.0),
                        rotation: Quat::from_rotation_z(angle),
                        ..Default::default()
                    },
                ))
                .id();

            if let Ok(map_entity) = map.single() {
                commands.entity(map_entity).add_child(dash);
            }

            line.spawned += 1;
        }

        if line.spawned >= line.total_dashes {
            commands.entity(entity).despawn();
        }
    }
    Ok(())
}

fn enter_travel_state(
    trigger: Trigger<OnAdd, Traveling>,
    query: Query<Entity, With<ControlledPlayer>>,
    mut next_state: ResMut<NextState<PlayerState>>,
) -> Result {
    info!("start enter travel state");
    let Ok(_) = query.get(trigger.target()) else {
        return Ok(());
    };
    info!("is controlled player, setting travel state");
    next_state.set(PlayerState::Traveling);
    Ok(())
}

fn leave_travel_state(
    trigger: Trigger<OnRemove, Traveling>,
    query: Query<Entity, With<ControlledPlayer>>,
    mut next_state: ResMut<NextState<PlayerState>>,
) -> Result {
    info!("start leave travel state");
    let Ok(_) = query.get(trigger.target()) else {
        return Ok(());
    };
    info!("is controlled player, setting world state");
    next_state.set(PlayerState::World);
    Ok(())
}

fn spawn_travel_dashline(
    mut commands: Commands,
    traveling: Query<&Traveling, With<ControlledPlayer>>,
) -> Result {
    let traveling = traveling.single()?;
    let source = traveling.source;
    let target = traveling.target;

    let dash_len = 4.5;
    let gap = 3.0;
    let thickness = 2.0;
    let color = Color::srgb_u8(206, 164, 129);
    let wiggle = 40.;
    let total_time = 5.;

    commands.spawn((DashLine::new(
        source.position,
        target.position,
        dash_len,
        gap,
        thickness,
        wiggle,
        color,
        total_time,
    ),));
    Ok(())
}

fn sync_ui_to_camera(
    mut query: Query<&mut Transform, With<UIElement>>,
    camera: Query<&Transform, (With<Camera>, Without<UIElement>)>,
) -> Result {
    if let Ok(camera) = camera.single() {
        for mut transform in &mut query.iter_mut() {
            transform.translation = camera.translation.with_z(100.);
        }
    }
    Ok(())
}

fn toggle_map(
    mut map: Query<&mut Visibility, With<Map>>,
    mut next_state: ResMut<NextState<PlayerState>>,
) -> Result {
    info!("toggle map");
    let mut map = map.single_mut()?;

    map.toggle_visible_hidden();

    if let Visibility::Hidden = *map {
        next_state.set(PlayerState::World);
    } else {
        next_state.set(PlayerState::Interaction);
    }
    Ok(())
}

fn show_map(mut map: Query<&mut Visibility, With<Map>>) -> Result {
    info!("show map");
    let mut map = map.single_mut()?;
    *map = Visibility::Visible;
    Ok(())
}

fn hide_map(mut map: Query<&mut Visibility, With<Map>>) -> Result {
    info!("hide map");
    let mut map = map.single_mut()?;
    *map = Visibility::Hidden;
    Ok(())
}
