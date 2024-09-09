use bevy::prelude::*;

use crate::shared::networking::{Facing, Movement};

use super::networking::UnitEvent;

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

#[derive(PartialEq, Eq, Debug)]
pub enum AnimationsState {
    Idle,
    Walk,
    Attack,
}

#[derive(Component)]
pub struct Animations {
    pub idle: (Handle<TextureAtlasLayout>, Timer),
    pub walk: (Handle<TextureAtlasLayout>, Timer),
    pub attack: (Handle<TextureAtlasLayout>, Timer),
}

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component)]
pub struct CurrentAnimation {
    pub state: AnimationsState,
    pub frame_timer: Timer,
    pub animation_duration: Timer,
}

#[derive(Event)]
struct AnimationChanged;

fn set_current_animation(
    mut unit_events: EventReader<UnitEvent>,
    mut query: Query<(
        Entity,
        &mut CurrentAnimation,
        &Movement,
        &Animations,
        &AnimationIndices,
    )>,
    mut change_animation: EventWriter<AnimationChanged>,
) {
    for event in unit_events.read() {
        for (entity, mut current_animation, _, animation, animation_indices) in &mut query {
            match event {
                UnitEvent::MeleeAttack(attacking_entity) => {
                    if entity == *attacking_entity {
                        current_animation.state = AnimationsState::Attack;

                        current_animation.animation_duration = Timer::new(
                            animation.attack.1.duration().mul_f32(
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
        &Animations,
        &Movement,
        &mut TextureAtlas,
        &mut Transform,
        &mut CurrentAnimation,
    )>,
    mut animation_changed: EventReader<AnimationChanged>,
) {
    for _state_change in animation_changed.read() {
        for (animations, movement, mut atlas, mut transform, mut current_animation) in &mut query {
            match current_animation.state {
                AnimationsState::Idle => {
                    current_animation.frame_timer = animations.idle.1.clone();
                    atlas.layout = animations.idle.0.clone();
                }
                AnimationsState::Walk => {
                    atlas.layout = animations.walk.0.clone();
                    current_animation.frame_timer = animations.walk.1.clone();
                }
                AnimationsState::Attack => {
                    atlas.layout = animations.attack.0.clone();
                    current_animation.frame_timer = animations.attack.1.clone();
                }
            }
            match movement.facing {
                Facing::Left => {
                    transform.scale.x = transform.scale.x.abs() * -1.;
                }
                Facing::Right => {
                    transform.scale.x = transform.scale.x.abs() * 1.;
                }
            }
        }
    }
}
