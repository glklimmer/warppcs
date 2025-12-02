use bevy::prelude::*;

use crate::Highlighted;

pub fn add_highlight_on<E: Clone + Reflect>(trigger: On<E>, mut commands: Commands) {
    let entity = trigger.target();
    commands.entity(entity).try_insert(Highlighted);
}

pub fn remove_highlight_on<E: Clone + Reflect>(trigger: On<E>, mut commands: Commands) {
    let entity = trigger.target();
    commands.entity(entity).try_remove::<Highlighted>();
}
