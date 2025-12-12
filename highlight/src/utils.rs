use bevy::prelude::*;

use crate::Highlighted;

pub fn add_highlight_on<E: EntityEvent>(trigger: On<E>, mut commands: Commands) {
    let target = trigger.event_target();
    commands.entity(target).try_insert(Highlighted);
}

pub fn remove_highlight_on<E: EntityEvent>(trigger: On<E>, mut commands: Commands) {
    let entity = trigger.event_target();
    commands.entity(entity).try_remove::<Highlighted>();
}
