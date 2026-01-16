use bevy::prelude::*;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{AppRuleExt, Replicated};
use health::Health;
use inventory::Inventory;
use physics::movement::{BoxCollider, Speed};
use serde::{Deserialize, Serialize};
use shared::{GameSceneId, GameState, Owner, Vec3LayerExt, map::Layers};
use std::collections::HashMap;
use transport::{HomeBuilding, Transport};

use crate::{BuildStatus, Building};

pub(crate) mod animation;

pub(crate) struct TransportPlugins;

impl Plugin for TransportPlugins {
    fn build(&self, app: &mut App) {
        app.replicate_bundle::<(TransportBuilding, Transform)>()
            .add_systems(
                FixedUpdate,
                (send_transporter.run_if(in_state(GameState::GameSession)),),
            );
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = marker_collider(),
    Sprite,
    Anchor::BOTTOM_CENTER,
    BuildStatus = BuildStatus::Marker,
)]
pub struct TransportBuilding {
    transporters: u8,
    collection_queue: Vec<Entity>,
}

impl Default for TransportBuilding {
    fn default() -> Self {
        Self {
            transporters: 1,
            collection_queue: vec![],
        }
    }
}

fn marker_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(28., 26.),
        offset: Some(Vec2::new(0., 13.)),
    }
}

fn send_transporter(
    mut commands: Commands,
    mut transport_building_query: Query<(
        Entity,
        &mut TransportBuilding,
        &Owner,
        &Transform,
        &BuildStatus,
        &GameSceneId,
    )>,
    transporter_query: Query<&HomeBuilding>,
    collectables_query: Query<(Entity, &Owner), (With<Building>, With<Inventory>)>,
) {
    let mut transport_counts = HashMap::new();
    for home_building in transporter_query.iter() {
        *transport_counts.entry(**home_building).or_default() += 1;
    }

    for (
        transport_building_entity,
        mut transport_building,
        transport_building_owner,
        transport_building_transform,
        build_status,
        game_scene_id,
    ) in transport_building_query.iter_mut()
    {
        let BuildStatus::Built { indicator: _ } = build_status else {
            continue;
        };

        if transport_building.collection_queue.is_empty() {
            transport_building.collection_queue = collectables_query
                .iter()
                .filter(|(_, owner)| *owner == transport_building_owner)
                .map(|(entity, _)| entity)
                .collect();
        }

        let active_transports_count = transport_counts
            .get(&transport_building_entity)
            .copied()
            .unwrap_or(0_u8);

        if active_transports_count < transport_building.transporters
            && let Some(target) = transport_building.collection_queue.pop()
        {
            info!("spawning transport");
            commands.spawn((
                Transport { target },
                HomeBuilding(transport_building_entity),
                *transport_building_owner,
                Health { hitpoints: 100.0 },
                Speed(100.0),
                transport_building_transform
                    .translation
                    .with_layer(Layers::Unit),
                *game_scene_id,
            ));
        }
    }
}
