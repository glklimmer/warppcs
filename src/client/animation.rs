use bevy::prelude::*;

use crate::shared::networking::{Facing, Movement};

use super::{
    generals::{AnimationsState, CurrentAnimation, GeneralAnimations},
    networking::UnitEvent,
};

#[derive(Event)]
struct AnimationChanged;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationChanged>();
        app.add_systems(
            Update,
            (
                animate,
                set_current_animation,
                animate_sprite_system,
                mirror_sprite,
            ),
        );
    }
}

fn set_current_animation(
    mut unit_events: EventReader<UnitEvent>,
    mut query: Query<(Entity, &mut CurrentAnimation, &Movement, &GeneralAnimations)>,
    mut change_animation: EventWriter<AnimationChanged>,
) {
    for event in unit_events.read() {
        for (entity, mut current_animation, _, general_animations) in &mut query {
            match event {
                UnitEvent::MeleeAttack(attacking_entity) => {
                    if entity == *attacking_entity {
                        current_animation.state = AnimationsState::Attack;
                        current_animation.current = general_animations.attack.clone();
                        change_animation.send(AnimationChanged);
                    }
                }
            }
        }
    }

    for (_, mut current_animation, movement, _) in &mut query {
        match current_animation.state {
            AnimationsState::Idle => {
                if movement.moving {
                    current_animation.state = AnimationsState::Walk;
                    change_animation.send(AnimationChanged);
                }
            }
            AnimationsState::Walk => {
                if !movement.moving {
                    current_animation.state = AnimationsState::Idle;
                    change_animation.send(AnimationChanged);
                }
            }
            AnimationsState::Attack => {
                if current_animation.current.animation_duration.finished() {
                    current_animation.state = match movement.moving {
                        true => AnimationsState::Walk,
                        false => AnimationsState::Idle,
                    };
                    change_animation.send(AnimationChanged);
                }
            }
        }
    }
}

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(&mut CurrentAnimation, &mut TextureAtlas)>,
) {
    for (mut current_animation, mut atlas) in &mut query {
        current_animation.current.frame_timer.tick(time.delta());
        current_animation
            .current
            .animation_duration
            .tick(time.delta());

        if current_animation.current.frame_timer.just_finished() {
            atlas.index = if atlas.index == current_animation.current.last_sprite_index {
                current_animation.current.first_sprite_index
            } else {
                atlas.index - 1
            };
        }
    }
}

fn animate(
    mut query: Query<(&GeneralAnimations, &mut TextureAtlas, &mut CurrentAnimation)>,
    mut animation_changed: EventReader<AnimationChanged>,
) {
    for _state_change in animation_changed.read() {
        for (animations, mut atlas, mut current_animation) in &mut query {
            // Update animation state
            let (new_layout, new_frame_timer) = match current_animation.state {
                AnimationsState::Idle => (
                    &animations.idle.layout_handle,
                    animations.idle.frame_timer.clone(),
                ),
                AnimationsState::Walk => (
                    &animations.walk.layout_handle,
                    animations.walk.frame_timer.clone(),
                ),
                AnimationsState::Attack => (
                    &animations.attack.layout_handle,
                    animations.attack.frame_timer.clone(),
                ),
            };

            // Update only if the animation has changed
            if atlas.layout != *new_layout {
                atlas.layout = new_layout.clone();
                current_animation.current.frame_timer = new_frame_timer;
            }
        }
    }
}

fn mirror_sprite(mut query: Query<(&Movement, &mut Transform)>) {
    for (movement, mut transform) in &mut query {
        let new_scale_x = match movement.facing {
            Facing::Left => -transform.scale.x.abs(),
            Facing::Right => transform.scale.x.abs(),
        };
        if transform.scale.x != new_scale_x {
            transform.scale.x = new_scale_x;
        }
    }
}
