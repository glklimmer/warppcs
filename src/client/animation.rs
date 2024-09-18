use bevy::prelude::*;

use crate::shared::networking::Facing;

use super::{
    king::{AnimationReferences, AnimationSetting},
    networking::{Change, NetworkEvent},
};

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub enum UnitAnimation {
    Idle,
    Walk,
    Attack,
}

#[derive(Component)]
pub struct UnitFacing(pub Facing);

#[derive(Debug, Event)]
/// Gets only triggered if new animation
pub struct AnimationTrigger {
    pub entity: Entity,
    pub state: UnitAnimation,
}

#[derive(Component)]
struct FullAnimation;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationTrigger>();

        app.add_systems(
            Update,
            (
                advance_animation,
                set_animation_settings,
                set_next_animation,
                set_unit_facing,
                mirror_sprite,
            ),
        );
    }
}

fn advance_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut AnimationSetting,
        &mut TextureAtlas,
        Option<&FullAnimation>,
    )>,
) {
    for (entity, mut current_animation, mut atlas, maybe_full) in &mut query {
        current_animation.config.frame_timer.tick(time.delta());

        if current_animation.config.frame_timer.just_finished() {
            atlas.index = if atlas.index == current_animation.config.last_sprite_index {
                if maybe_full.is_some() {
                    commands.entity(entity).remove::<FullAnimation>();
                }
                current_animation.config.first_sprite_index
            } else {
                atlas.index - 1
            };
        }
    }
}

fn set_next_animation(
    mut commands: Commands,
    mut animation: Query<(&mut AnimationSetting, Option<&FullAnimation>)>,
    mut network_events: EventReader<NetworkEvent>,
    mut animation_trigger: EventWriter<AnimationTrigger>,
) {
    for event in network_events.read() {
        if let Ok((mut current_animation, maybe_full)) = animation.get_mut(event.entity) {
            let new_animation = match &event.change {
                Change::Movement(movement) => match movement.moving {
                    true => UnitAnimation::Walk,
                    false => UnitAnimation::Idle,
                },
                Change::Attack => UnitAnimation::Attack,
            };

            if is_interupt_animation(&new_animation)
                || (maybe_full.is_none() && new_animation != current_animation.state)
            {
                current_animation.state = new_animation;

                if is_full_animation(&new_animation) {
                    commands.entity(event.entity).insert(FullAnimation);
                }
                animation_trigger.send(AnimationTrigger {
                    entity: event.entity,
                    state: new_animation,
                });

                if is_full_animation(&new_animation) {
                    break;
                }
            }
        }
    }
}

fn is_interupt_animation(animation: &UnitAnimation) -> bool {
    match animation {
        UnitAnimation::Idle => false,
        UnitAnimation::Walk => false,
        UnitAnimation::Attack => true,
    }
}

fn is_full_animation(animation: &UnitAnimation) -> bool {
    match animation {
        UnitAnimation::Idle => false,
        UnitAnimation::Walk => false,
        UnitAnimation::Attack => true,
    }
}

fn set_unit_facing(mut commands: Commands, mut movements: EventReader<NetworkEvent>) {
    for event in movements.read() {
        if let Change::Movement(movement) = &event.change {
            if let Some(new_facing) = &movement.facing {
                commands
                    .entity(event.entity)
                    .insert(UnitFacing(new_facing.clone()));
            }
        }
    }
}

fn set_animation_settings(
    mut query: Query<
        (
            &AnimationReferences,
            &mut AnimationSetting,
            &mut TextureAtlas,
        ),
        Changed<AnimationSetting>,
    >,
    mut animation_changed: EventReader<AnimationTrigger>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((references, mut current_animation, mut atlas)) =
            query.get_mut(new_animation.entity)
        {
            let reference = match current_animation.state {
                UnitAnimation::Idle => &references.idle,
                UnitAnimation::Walk => &references.walk,
                UnitAnimation::Attack => &references.attack,
            };

            atlas.layout = reference.layout_handle.clone();
            atlas.index = reference.first_sprite_index;

            current_animation.config.frame_timer = reference.frame_timer.clone();
            current_animation.config.frame_timer.reset();
        }
    }
}

fn mirror_sprite(mut query: Query<(&UnitFacing, &mut Transform)>) {
    for (unit_facing, mut transform) in &mut query {
        let new_scale_x = match unit_facing.0 {
            Facing::Left => -transform.scale.x.abs(),
            Facing::Right => transform.scale.x.abs(),
        };
        if transform.scale.x != new_scale_x {
            transform.scale.x = new_scale_x;
        }
    }
}
