use bevy::prelude::*;

use shared::networking::{Facing, Rotation, ServerMessages};

use crate::networking::{NetworkEvent, NetworkMapping};

use super::king::{AnimationConfig, AnimationReferences, AnimationSetting};

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

pub enum Change {
    Rotation(Rotation),
    Movement(bool),
    Attack,
}

#[derive(Event)]
pub struct EntityChangeEvent {
    pub entity: Entity,
    pub change: Change,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationTrigger>();
        app.add_event::<EntityChangeEvent>();

        app.add_systems(FixedUpdate, trigger_meele_attack);

        app.add_systems(
            Update,
            (
                advance_animation_minimal,
                advance_animation,
                set_animation_settings,
                set_next_animation,
                set_unit_facing,
                set_free_orientation,
                mirror_sprite,
            ),
        );
    }
}

fn trigger_meele_attack(
    mut network_events: EventReader<NetworkEvent>,
    mut change: EventWriter<EntityChangeEvent>,
    network_mapping: Res<NetworkMapping>,
) {
    for event in network_events.read() {
        if let ServerMessages::MeleeAttack {
            entity: server_entity,
        } = event.message
        {
            if let Some(client_entity) = network_mapping.0.get(&server_entity) {
                change.send(EntityChangeEvent {
                    entity: *client_entity,
                    change: Change::Attack,
                });
            }
        }
    }
}

fn advance_animation_minimal(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut AnimationConfig,
        &mut TextureAtlas,
        Option<&FullAnimation>,
    )>,
) {
    for (entity, mut animation_config, mut atlas, maybe_full) in &mut query {
        animation_config.frame_timer.tick(time.delta());

        if animation_config.frame_timer.just_finished() {
            atlas.index = if atlas.index == animation_config.last_sprite_index {
                if maybe_full.is_some() {
                    commands.entity(entity).remove::<FullAnimation>();
                }
                animation_config.first_sprite_index
            } else {
                atlas.index - 1
            };
        }
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
    mut network_events: EventReader<EntityChangeEvent>,
    mut animation_trigger: EventWriter<AnimationTrigger>,
) {
    for event in network_events.read() {
        if let Ok((mut current_animation, maybe_full)) = animation.get_mut(event.entity) {
            let maybe_new_animation = match &event.change {
                Change::Movement(moving) => match moving {
                    true => Some(UnitAnimation::Walk),
                    false => Some(UnitAnimation::Idle),
                },
                Change::Attack => Some(UnitAnimation::Attack),
                Change::Rotation(_) => None,
            };

            if let Some(new_animation) = maybe_new_animation {
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

fn set_unit_facing(mut commands: Commands, mut movements: EventReader<EntityChangeEvent>) {
    for event in movements.read() {
        if let Change::Rotation(Rotation::LeftRight {
            facing: Some(new_facing),
        }) = &event.change
        {
            if let Some(mut entity) = commands.get_entity(event.entity) {
                entity.try_insert(UnitFacing(new_facing.clone()));
            }
        }
    }
}

fn set_free_orientation(
    mut query: Query<&mut Transform>,
    mut movements: EventReader<EntityChangeEvent>,
) {
    for event in movements.read() {
        if let Change::Rotation(Rotation::Free { angle }) = &event.change {
            if let Ok(mut transform) = query.get_mut(event.entity) {
                transform.rotation = Quat::from_axis_angle(Vec3::Z, *angle);
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
