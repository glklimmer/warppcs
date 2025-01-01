use bevy::prelude::*;

use crate::{networking::MultiplayerRoles, GameState};

#[derive(Component)]
pub struct AttachedTo(pub Entity);

pub struct AttachmentPlugin;

impl Plugin for AttachmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (attachment_follow)
                .run_if(in_state(GameState::GameSession).and(in_state(MultiplayerRoles::Host))),
        );
    }
}

fn attachment_follow(
    mut query: Query<(&AttachedTo, &mut Transform)>,
    target: Query<&Transform, Without<AttachedTo>>,
) {
    for (item, mut item_transform) in query.iter_mut() {
        let target = target.get(item.0).unwrap();
        item_transform.translation = target.translation;
    }
}
