use bevy::prelude::*;

use crate::shared::networking::{Facing, Movement};

use super::{
    generals::{AnimationIndices, AnimationsState, CurrentAnimation, GeneralAnimations},
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
            (animate, set_current_animation, animate_sprite_system),
        );
    }
}

fn set_current_animation(
    mut unit_events: EventReader<UnitEvent>,
    mut query: Query<(
        Entity,
        &mut CurrentAnimation,
        &Movement,
        &AnimationIndices,
        &GeneralAnimations,
    )>,
    mut change_animation: EventWriter<AnimationChanged>,
) {
    for event in unit_events.read() {
        for (entity, mut current_animation, _, animation_indices, general_animations) in &mut query
        {
            match event {
                UnitEvent::MeleeAttack(attacking_entity) => {
                    if entity == *attacking_entity {
                        current_animation.state = AnimationsState::Attack;
                        current_animation.animation_duration = Timer::new(
                            general_animations.attack.1.duration().mul_f32(
                                animation_indices.last.abs_diff(animation_indices.first) as f32
                                    + 1.0,
                            ),
                            TimerMode::Once,
                        );
                        change_animation.send(AnimationChanged);
                    }
                }
            }
        }
    }

    for (_, mut current_animation, movement, _, _) in &mut query {
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
                if current_animation.animation_duration.finished() {
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
    mut query: Query<(&AnimationIndices, &mut CurrentAnimation, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.frame_timer.tick(time.delta());
        timer.animation_duration.tick(time.delta());

        if timer.frame_timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index - 1
            };
        }
    }
}

fn animate(
    mut query: Query<(
        &GeneralAnimations,
        &Movement,
        &mut TextureAtlas,
        &mut Transform,
        &mut CurrentAnimation,
    )>,
    mut animation_changed: EventReader<AnimationChanged>,
) {
    for _state_change in animation_changed.read() {
        for (animations, movement, mut atlas, mut transform, mut current_animation) in &mut query {
            // Check if facing direction has changed
            let new_scale_x = match movement.facing {
                Facing::Left => -transform.scale.x.abs(),
                Facing::Right => transform.scale.x.abs(),
            };

            if transform.scale.x != new_scale_x {
                transform.scale.x = new_scale_x;
                println!("{}", if new_scale_x < 0.0 { "Left" } else { "Right" });
            }

            // Update animation state
            let (new_layout, new_frame_timer) = match current_animation.state {
                AnimationsState::Idle => {
                    println!("Idle");
                    (&animations.idle.0, animations.idle.1.clone())
                }
                AnimationsState::Walk => {
                    println!("Walking");
                    (&animations.walk.0, animations.walk.1.clone())
                }
                AnimationsState::Attack => {
                    println!("Attack");
                    (&animations.attack.0, animations.attack.1.clone())
                }
            };

            // Update only if the animation has changed
            if atlas.layout != *new_layout {
                atlas.layout = new_layout.clone();
                current_animation.frame_timer = new_frame_timer;
            }
        }
    }
}
