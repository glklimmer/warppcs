use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize)]
pub struct AttachedTo(pub Entity);

impl MapEntities for AttachedTo {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0)
    }
}

pub struct AttachmentPlugin;

impl Plugin for AttachmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, attachment_follow);
    }
}

fn attachment_follow(
    mut query: Query<(&AttachedTo, &mut Transform)>,
    target: Query<&Transform, Without<AttachedTo>>,
) {
    for (item, mut item_transform) in query.iter_mut() {
        if let Ok(target) = target.get(item.0) {
            item_transform.translation.x = target.translation.x;
            item_transform.translation.y = target.translation.y + 2.;
        }
    }
}
