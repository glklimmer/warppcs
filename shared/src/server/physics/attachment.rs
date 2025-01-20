use bevy::prelude::*;

#[derive(Component)]
pub struct AttachedTo(pub Entity);

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
        let target = target.get(item.0).unwrap();
        item_transform.translation = target.translation;
    }
}
