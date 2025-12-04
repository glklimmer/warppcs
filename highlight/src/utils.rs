use bevy::prelude::*;

use crate::Highlighted;

pub fn add_highlight_on<E: Clone + Reflect + Event>(trigger: On<E>, mut commands: Commands) {
    let target = trigger.observer();
    commands.entity(target).try_insert(Highlighted);
}

pub fn remove_highlight_on<E: Clone + Reflect + Event>(trigger: On<E>, mut commands: Commands) {
    let entity = trigger.observer();
    commands.entity(entity).try_remove::<Highlighted>();
}
