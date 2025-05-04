use std::f32::consts::TAU;

use bevy::prelude::*;

use bevy::input::common_conditions::input_just_pressed;
use bevy::ui::prelude::{ImageNode, PositionType, Val};
use bevy_replicon::prelude::FromClient;
use shared::{Player, networking::LobbyEvent};

use crate::animations::ui::map_icon::{MapIconSpriteSheet, MapIcons};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                setup_map,
                toggle_map.run_if(input_just_pressed(KeyCode::KeyM)),
            ),
        );
    }
}

#[derive(Component)]
struct Map;

fn setup_map(
    mut lobby_events: EventReader<FromClient<LobbyEvent>>,
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    assets: Res<AssetServer>,
    map_icons: Res<MapIconSpriteSheet>,
) {
    let Some(FromClient { event, .. }) = lobby_events.read().next() else {
        return;
    };
    #[allow(irrefutable_let_patterns)]
    let LobbyEvent::StartGame = event else {
        return;
    };

    let map_texture = assets.load("sprites/ui/map.png");

    let total_icons = players.iter().count() * 2;
    let radius = 75.0;

    commands
        .spawn((
            Map,
            Visibility::Hidden,
            ImageNode {
                image: map_texture.clone(),
                ..Default::default()
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                // center children
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Percent(50.0),
                    ..Default::default()
                })
                .with_children(|parent| {
                    for i in 0..total_icons {
                        let angle = (i as f32 / total_icons as f32) * TAU;
                        let x = radius * angle.cos();
                        let y = radius * angle.sin();

                        let icon_type = if i % 2 == 0 {
                            MapIcons::Player
                        } else {
                            MapIcons::Bandit
                        };

                        parent.spawn((
                            ImageNode::from_atlas_image(
                                map_icons.sprite_sheet.texture.clone(),
                                map_icons.sprite_sheet.texture_atlas(icon_type),
                            ),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(x),
                                bottom: Val::Px(y),
                                ..Default::default()
                            },
                        ));
                    }
                });
        });
}

fn toggle_map(mut map: Query<&mut Visibility, With<Map>>) {
    info!("toggle map");
    let Ok(mut map) = map.get_single_mut() else {
        return;
    };

    map.toggle_visible_hidden();
}
