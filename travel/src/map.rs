use bevy::{platform::collections::HashMap, prelude::*};

use animations::ui::map_icon::{MapIconSpriteSheet, MapIcons};
use bevy::input::common_conditions::input_just_pressed;
use bevy_replicon::prelude::{
    Channel, ClientEventAppExt, ClientId, ClientState, ClientTriggerExt, SendMode,
    ServerEventAppExt, ServerTriggerExt, ToClients,
};
use highlight::{
    Highlightable,
    utils::{add_highlight_on, remove_highlight_on},
};
use serde::{Deserialize, Serialize};
use shared::{
    ClientPlayerMap, ControlledPlayer, GameScene, GameState, PlayerState, SceneType,
    SetLocalPlayer,
    server::players::interaction::{InteractionTriggeredEvent, InteractionType},
};

use crate::{TravelDestinations, Traveling};

pub struct TravelPlugin;

impl Plugin for TravelPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_event::<SelectTravelDestination>(Channel::Ordered)
            .add_server_event::<OpenTravelDialog>(Channel::Ordered)
            .insert_state(MapState::View)
            .add_observer(init_map)
            .add_observer(map_discovery_change)
            .add_observer(open_travel_dialog)
            .add_systems(OnEnter(PlayerState::Traveling), spawn_travel_dashline)
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
            .add_systems(
                FixedUpdate,
                (init_travel_dialog).run_if(in_state(ClientState::Disconnected)),
            );
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MapDiscovery {
    game_scenes: HashMap<GameScene, DiscoveryType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum DiscoveryType {
    Unrevealed,
    Revealed,
}

#[derive(Event, Debug, Serialize, Deserialize, Clone)]
struct DiscoveryChange {
    game_scene: GameScene,
    change_type: DiscoveryType,
}

impl MapDiscovery {
    pub fn set_base(&mut self, mut commands: Commands, client: ClientId, base: GameScene) {
        self.game_scenes.insert(base, DiscoveryType::Revealed);
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(client),
            message: DiscoveryChange {
                game_scene: base,
                change_type: DiscoveryType::Revealed,
            },
        });
    }
    pub fn add_unrevealed(
        &mut self,
        mut commands: Commands,
        client: ClientId,
        game_scene: GameScene,
    ) {
        self.game_scenes
            .insert(game_scene, DiscoveryType::Unrevealed);

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(client),
            message: DiscoveryChange {
                game_scene,
                change_type: DiscoveryType::Revealed,
            },
        });
    }

    pub fn reveal(
        &mut self,
        mut commands: Commands,
        client: ClientId,
        game_scene: GameScene,
    ) -> Result {
        let discovery_type = self
            .game_scenes
            .get_mut(&game_scene)
            .ok_or("Trying to reveal unknown GameScene")?;
        *discovery_type = DiscoveryType::Revealed;

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(client),
            message: DiscoveryChange {
                game_scene,
                change_type: DiscoveryType::Revealed,
            },
        });
        Ok(())
    }
}

#[derive(Event, Deserialize, Serialize)]
struct OpenTravelDialog {
    current_scene: GameScene,
}

#[derive(Event, Deserialize, Serialize, Deref)]
pub(crate) struct SelectTravelDestination(pub GameScene);

#[derive(Component, Deref)]
struct MapNode(GameScene);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum MapState {
    View,
    Selection { current_scene: GameScene },
}

#[derive(Component)]
struct DashLine {
    a: Vec2,
    b: Vec2,
    dash_len: f32,
    thickness: f32,
    color: Color,

    cp1: Vec2,
    cp2: Vec2,
    total_dashes: usize,
    spawned: usize,
    timer: Timer,
}

#[derive(Component, Default)]
struct UIElement;

#[derive(Component)]
#[require(UIElement)]
struct Map;

fn init_travel_dialog(
    mut traveling: MessageReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    destinations: Query<&TravelDestinations>,
    game_scene: Query<&GameScene>,
    mut discovery: Query<&mut MapDiscovery>,
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

        let mut discovery = discovery.get_mut(player_entity)?;
        for destination in &**destinations {
            let game_scene = game_scene.get(*destination)?;
            discovery.add_unrevealed(commands.reborrow(), *client, *game_scene);
        }

        let current_scene = game_scene.get(event.interactable)?;

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(*client),
            message: OpenTravelDialog {
                current_scene: *current_scene,
            },
        });
    }
    Ok(())
}

fn init_map(
    _trigger: On<SetLocalPlayer>,
    assets: Res<AssetServer>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) -> Result {
    let map_texture = assets.load::<Image>("sprites/ui/map.png");

    next_game_state.set(GameState::GameSession);

    commands.spawn((
        Map,
        Visibility::Hidden,
        Sprite::from_image(map_texture),
        Transform::from_scale(Vec3::splat(1.0 / 3.0)),
    ));
    Ok(())
}

fn open_travel_dialog(
    trigger: On<OpenTravelDialog>,
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
    trigger: On<Pointer<Release>>,
    mut commands: Commands,
    map_state: ResMut<State<MapState>>,
    mut next_map_state: ResMut<NextState<MapState>>,
    map_node: Query<&MapNode>,
) -> Result {
    let MapState::Selection { current_scene } = map_state.get() else {
        return Ok(());
    };

    let entity = trigger.entity;
    let game_scene = **map_node.get(entity)?;

    if game_scene.eq(current_scene) {
        return Ok(());
    }

    next_map_state.set(MapState::View);

    commands.client_trigger(SelectTravelDestination(game_scene));
    Ok(())
}

fn map_discovery_change(
    trigger: On<DiscoveryChange>,
    map: Query<Entity, With<Map>>,
    map_icons: Res<MapIconSpriteSheet>,
    query: Query<(Entity, &MapNode)>,
    mut commands: Commands,
) -> Result {
    let map = map.single()?;

    let change = trigger.event();
    let game_scene = change.game_scene;

    let icon = match change.change_type {
        DiscoveryType::Unrevealed => MapIcons::Mystery,
        DiscoveryType::Revealed => match game_scene.scene {
            SceneType::Player { .. } => MapIcons::Player,
            SceneType::Camp { .. } => MapIcons::Bandit,
            SceneType::Meadow { .. } => MapIcons::Bandit,
        },
    };

    match query.iter().find(|(_, node)| node.eq(&game_scene)) {
        Some((entity, _)) => {
            commands.entity(entity).insert(Sprite::from_atlas_image(
                map_icons.sprite_sheet.texture.clone(),
                map_icons.sprite_sheet.texture_atlas(icon),
            ));
        }
        None => {
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
        }
    }

    Ok(())
}

impl DashLine {
    fn new(
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

        if line.timer.is_finished() && line.spawned < line.total_dashes {
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

fn hide_map(mut map: Query<&mut Visibility, With<Map>>) -> Result {
    let mut map = map.single_mut()?;
    *map = Visibility::Hidden;
    Ok(())
}
