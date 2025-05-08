use bevy::prelude::*;

use bevy::input::common_conditions::input_just_pressed;
use shared::{
    PlayerState,
    server::game_scenes::map::{LoadMap, NodeType},
};

use crate::animations::ui::map_icon::{MapIconSpriteSheet, MapIcons};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_map).add_systems(
            Update,
            (
                sync_ui_elements,
                toggle_map.run_if(input_just_pressed(KeyCode::KeyM)),
            ),
        );
    }
}

#[derive(Component, Default)]
struct UIElement;

fn sync_ui_elements(
    mut query: Query<&mut Transform, With<UIElement>>,
    camera: Query<&Transform, (With<Camera>, Without<UIElement>)>,
) {
    let Ok(camera) = camera.get_single() else {
        return;
    };

    for mut transform in &mut query.iter_mut() {
        transform.translation = camera.translation.with_z(100.);
    }
}

#[derive(Component)]
#[require(UIElement)]
struct Map;

fn setup_map(
    trigger: Trigger<LoadMap>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    map_icons: Res<MapIconSpriteSheet>,
) {
    let map = &**trigger.event();
    let map_texture = assets.load::<Image>("sprites/ui/map.png");

    commands
        .spawn((
            Map,
            Visibility::Hidden,
            Sprite::from_image(map_texture),
            Transform::from_scale(Vec3::new(1. / 3., 1. / 3., 1.)),
        ))
        .with_children(|parent| {
            for node in &map.nodes {
                let icon_type = match node.node_type {
                    NodeType::Player => MapIcons::Player,
                    NodeType::Bandit => MapIcons::Bandit,
                };

                parent.spawn((
                    Sprite::from_atlas_image(
                        map_icons.sprite_sheet.texture.clone(),
                        map_icons.sprite_sheet.texture_atlas(icon_type),
                    ),
                    Transform::from_xyz(node.position.x, node.position.y, 0.1),
                ));
            }
        });
}

fn toggle_map(
    mut map: Query<&mut Visibility, With<Map>>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    info!("toggle map");
    let Ok(mut map) = map.get_single_mut() else {
        return;
    };

    map.toggle_visible_hidden();

    if let Visibility::Hidden = *map {
        next_state.set(PlayerState::World);
    } else {
        next_state.set(PlayerState::Interaction);
    }
}
