use bevy::prelude::*;

use crate::shared::networking::{Facing, Movement};

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
                // attack_system,
            ),
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
    pub timer: Timer,
}

#[derive(Event)]
struct AnimationChanged;

fn set_current_animation(
    mut query: Query<(&mut CurrentAnimation, &Movement)>,
    mut change_animation: EventWriter<AnimationChanged>,
) {
    for (mut animation_state, movement) in &mut query {
        match animation_state.state {
            AnimationsState::Idle => {
                if movement.moving {
                    animation_state.state = AnimationsState::Walk;
                    change_animation.send(AnimationChanged);
                }
            }
            AnimationsState::Walk => {
                if !movement.moving {
                    animation_state.state = AnimationsState::Idle;
                    change_animation.send(AnimationChanged);
                }
            }
            AnimationsState::Attack => todo!(),
        }
    }
}

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut CurrentAnimation, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
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
                    current_animation.timer = animations.idle.1.clone();
                    atlas.layout = animations.idle.0.clone();
                }
                AnimationsState::Walk => {
                    atlas.layout = animations.walk.0.clone();
                    current_animation.timer = animations.walk.1.clone();
                    match movement.facing {
                        Facing::Left => {
                            transform.scale.x = transform.scale.x.abs() * -1.;
                        }
                        Facing::Right => {
                            transform.scale.x = transform.scale.x.abs() * 1.;
                        }
                    }
                }
                AnimationsState::Attack => todo!(),
            }
        }
    }
}

// fn attack_system(
//     kb: Res<ButtonInput<KeyCode>>,
//     mut query: Query<(&Animations, &mut TextureAtlas, &mut CurrentAnimation)>,
// ) {
//     for (animations, mut atlas, mut animation_timer) in &mut query {
//         if kb.pressed(KeyCode::KeyE) {
//             atlas.layout = animations.attack.0.clone();
//             if animation_timer.state != AnimationsState::Attack {
//                 animation_timer.state = AnimationsState::Attack;
//                 animation_timer.timer = animations.attack.1.clone();
//             }
//         }
//     }
// }
