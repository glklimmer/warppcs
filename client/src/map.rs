use bevy::prelude::*;

use bevy::input::common_conditions::input_just_pressed;

use shared::{
    PlayerState,
    server::game_scenes::{
        travel::Traveling,
        world::{InitPlayerMapNode, RevealMapNode, SceneType},
    },
};

use crate::{
    animations::ui::map_icon::{MapIconSpriteSheet, MapIcons},
    networking::ControlledPlayer,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_map)
            // ------------
            // todo: move to state(?) plugin
            .add_observer(enter_travel_state)
            .add_observer(leave_travel_state)
            // ------------
            .add_observer(reveal_map_icons)
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
                        .run_if(not(in_state(PlayerState::Traveling))),
                ),
            )
            .add_systems(Update, animate_dashes);
    }
}

#[derive(Component)]
struct MapIcon;

fn init_map(
    trigger: Trigger<InitPlayerMapNode>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    map_icons: Res<MapIconSpriteSheet>,
) {
    let player_scene = **trigger.event();
    let map_texture = assets.load::<Image>("sprites/ui/map.png");

    commands
        .spawn((
            Map,
            Visibility::Hidden,
            Sprite::from_image(map_texture),
            Transform::from_scale(Vec3::splat(1.0 / 3.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                MapIcon,
                Visibility::Inherited,
                Sprite::from_atlas_image(
                    map_icons.sprite_sheet.texture.clone(),
                    map_icons.sprite_sheet.texture_atlas(MapIcons::Player),
                ),
                Transform::from_xyz(player_scene.position.x, player_scene.position.y, 2.0),
            ));
        });
}

fn reveal_map_icons(
    trigger: Trigger<RevealMapNode>,
    mut commands: Commands,
    map: Query<Entity, With<Map>>,
    map_icons: Res<MapIconSpriteSheet>,
) {
    info!("reveal map node");
    let map_node = **trigger.event();
    let map = map.single().unwrap();
    let icon = match map_node.scene {
        SceneType::Player { .. } => MapIcons::Player,
        SceneType::Traversal { .. } => MapIcons::Bandit,
        SceneType::TJunction { .. } => MapIcons::Bandit,
        SceneType::DoubleConnection { .. } => MapIcons::Bandit,
    };

    commands.spawn((
        ChildOf(map),
        MapIcon,
        Visibility::Inherited,
        Sprite::from_atlas_image(
            map_icons.sprite_sheet.texture.clone(),
            map_icons.sprite_sheet.texture_atlas(icon),
        ),
        Transform::from_xyz(map_node.position.x, map_node.position.y, 2.0),
    ));
}

use fastrand::f32 as rand_f32;

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
        let t1 = rand_f32() * 0.5 + 0.1;
        let t2 = rand_f32() * 0.5 + 0.4;
        let cp1 = a.lerp(b, t1) + perp * (rand_f32() * 2.0 - 1.0) * wiggle;
        let cp2 = a.lerp(b, t2) + perp * (rand_f32() * 2.0 - 1.0) * wiggle;

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
) {
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

            commands.entity(map.single().unwrap()).add_child(dash);

            line.spawned += 1;
        }

        if line.spawned >= line.total_dashes {
            commands.entity(entity).despawn();
        }
    }
}

fn enter_travel_state(
    trigger: Trigger<OnAdd, Traveling>,
    query: Query<Entity, With<ControlledPlayer>>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    let Ok(_) = query.get(trigger.target()) else {
        return;
    };
    next_state.set(PlayerState::Traveling);
}

fn leave_travel_state(
    trigger: Trigger<OnRemove, Traveling>,
    query: Query<Entity, With<ControlledPlayer>>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    let Ok(_) = query.get(trigger.target()) else {
        return;
    };
    next_state.set(PlayerState::World);
}

fn spawn_travel_dashline(
    mut commands: Commands,
    traveling: Query<&Traveling, With<ControlledPlayer>>,
) {
    let traveling = traveling.single().unwrap();
    let (_, maybe_source_game_scene) = traveling.source;
    let (_, maybe_target_game_scene) = traveling.target;
    let Some(source) = maybe_source_game_scene else {
        return;
    };
    let Some(target) = maybe_target_game_scene else {
        return;
    };

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
}

#[derive(Component, Default)]
struct UIElement;

fn sync_ui_to_camera(
    mut query: Query<&mut Transform, With<UIElement>>,
    camera: Query<&Transform, (With<Camera>, Without<UIElement>)>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };

    for mut transform in &mut query.iter_mut() {
        transform.translation = camera.translation.with_z(100.);
    }
}

#[derive(Component)]
#[require(UIElement)]
struct Map;

fn toggle_map(
    mut map: Query<&mut Visibility, With<Map>>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    info!("toggle map");
    let Ok(mut map) = map.single_mut() else {
        return;
    };

    map.toggle_visible_hidden();

    if let Visibility::Hidden = *map {
        next_state.set(PlayerState::World);
    } else {
        next_state.set(PlayerState::Interaction);
    }
}

fn show_map(mut map: Query<&mut Visibility, With<Map>>) {
    if let Ok(mut map) = map.single_mut() {
        *map = Visibility::Visible;
    }
}

fn hide_map(mut map: Query<&mut Visibility, With<Map>>) {
    if let Ok(mut map) = map.single_mut() {
        *map = Visibility::Hidden;
    }
}
