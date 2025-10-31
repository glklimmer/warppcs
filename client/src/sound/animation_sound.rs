use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use shared::{
    AnimationChange, AnimationChangeEvent, Hitby,
    server::{
        physics::projectile::ProjectileType,
        players::interaction::{InteractableSound, InteractionType},
    },
};

use crate::animations::{AnimationSound, AnimationSoundTrigger, SpriteSheetAnimation};

#[derive(Event)]
struct PlayAnimationSoundEvent {
    entity: Entity,
    sound_handles: Vec<Handle<AudioSource>>,
    speed: f32,
    volume: Volume,
}

const ANIMATION_VOLUME: f32 = 0.25;

pub struct AnimationSoundPlugin;

impl Plugin for AnimationSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayAnimationSoundEvent>();
        app.add_observer(play_animation_on_projectile_spawn);
        app.add_observer(play_on_interactable);
        app.add_observer(stop_animation_sound_on_remove);

        app.add_systems(
            Update,
            (
                handle_single_animation_sound.run_if(on_event::<PlayAnimationSoundEvent>),
                handle_multiple_animation_sound.run_if(on_event::<PlayAnimationSoundEvent>),
                play_sound_on_entity_change,
                play_animation_on_frame_timer,
                play_animation_on_enter,
            ),
        );
    }
}

fn stop_animation_sound_on_remove(
    trigger: Trigger<OnRemove, AnimationSound>,
    query: Query<Option<&SpatialAudioSink>>,
    mut commands: Commands,
) -> Result {
    let maybe_sink = query.get(trigger.target())?;
    if let Some(sink) = maybe_sink {
        sink.stop();
        sink.pause();
    };
    commands.entity(trigger.target()).remove::<AudioPlayer>();
    Ok(())
}

fn handle_multiple_animation_sound(
    mut sound_events: EventReader<PlayAnimationSoundEvent>,
    mut commands: Commands,
) -> Result {
    for event in sound_events.read() {
        if event.sound_handles.is_empty() {
            continue;
        }

        let Some(random_sound) = fastrand::choice(event.sound_handles.iter()) else {
            return Err(BevyError::from("No sound handle prvided"));
        };
        if let Ok(mut entity_command) = commands.get_entity(event.entity) {
            entity_command.insert((
                AudioPlayer::<AudioSource>(random_sound.clone_weak()),
                PlaybackSettings {
                    mode: PlaybackMode::Remove,
                    speed: event.speed,
                    volume: event.volume,
                    spatial: true,
                    ..default()
                },
            ));
        }
    }
    Ok(())
}

fn handle_single_animation_sound(
    mut sound_events: EventReader<PlayAnimationSoundEvent>,
    mut commands: Commands,
) -> Result {
    for event in sound_events.read() {
        if event.sound_handles.len() != 1 {
            continue;
        };

        let mut entity_command = commands.get_entity(event.entity)?;
        entity_command.insert((
            AudioPlayer::<AudioSource>(event.sound_handles[0].clone_weak()),
            PlaybackSettings {
                mode: PlaybackMode::Remove,
                speed: event.speed,
                volume: event.volume,
                spatial: true,
                ..default()
            },
        ));
    }
    Ok(())
}

fn play_sound_on_entity_change(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    mut entity_change_events: EventReader<AnimationChangeEvent>,
    asset_server: Res<AssetServer>,
) -> Result {
    for event in entity_change_events.read() {
        let sound = match event.change {
            AnimationChange::Hit(hit_by) => match hit_by {
                Hitby::Arrow => "animation_sound/arrow/arrow_hits_flesh.ogg",
                Hitby::Melee => "animation_sound/arrow/arrow_hits_flesh.ogg",
            },
            AnimationChange::Mount => "animation_sound/horse/horse_sound.ogg",
            _ => continue,
        };

        sound_events.write(PlayAnimationSoundEvent {
            entity: event.entity,
            sound_handles: if sound.is_empty() {
                Vec::new()
            } else {
                vec![asset_server.load(sound)]
            },
            speed: 1.0,
            volume: Volume::Linear(ANIMATION_VOLUME),
        });
    }
    Ok(())
}

fn play_animation_on_projectile_spawn(
    trigger: Trigger<OnAdd, ProjectileType>,
    mut projectile: Query<&ProjectileType>,
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    asset_server: Res<AssetServer>,
) -> Result {
    let projectile_type = projectile.get_mut(trigger.target())?;

    let sound_handles = match projectile_type {
        ProjectileType::Arrow => vec![asset_server.load("animation_sound/arrow/arrow_flying.ogg")],
    };

    sound_events.write(PlayAnimationSoundEvent {
        entity: trigger.target(),
        sound_handles,
        speed: 1.5,
        volume: Volume::Linear(ANIMATION_VOLUME),
    });
    Ok(())
}

fn play_on_interactable(
    trigger: Trigger<InteractableSound>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) -> Result {
    let audio_file = match trigger.kind {
        InteractionType::Recruit => {
            asset_server.load("animation_sound/recruitment/recruite_call.ogg")
        }
        InteractionType::Flag => todo!(),
        InteractionType::Building => todo!(),
        InteractionType::Mount => todo!(),
        InteractionType::Travel => todo!(),
        InteractionType::Chest => todo!(),
        InteractionType::Item => todo!(),
        InteractionType::Commander => todo!(),
        InteractionType::ItemAssignment => todo!(),
        InteractionType::Unmount => todo!(),
        InteractionType::Portal => todo!(),
    };

    commands.spawn((
        AudioPlayer::<AudioSource>(audio_file),
        PlaybackSettings {
            mode: PlaybackMode::Remove,
            volume: Volume::Linear(ANIMATION_VOLUME * 0.5),
            spatial: true,
            ..default()
        },
        Transform::from_translation(trigger.spatial_position),
    ));
    Ok(())
}

fn play_animation_on_frame_timer(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    query: Query<(
        Entity,
        &SpriteSheetAnimation,
        Option<&AnimationSound>,
        &Sprite,
    )>,
) -> Result {
    for (entity, sprite_animation, animation, sprite) in query.iter() {
        let Some(sound) = &animation else {
            continue;
        };
        let AnimationSoundTrigger::OnStartFrameTimer = sound.sound_trigger else {
            continue;
        };

        let atlas = sprite.texture_atlas.as_ref().unwrap();
        if sprite_animation.frame_timer.just_finished()
            && atlas.index == sprite_animation.first_sprite_index
        {
            sound_events.write(PlayAnimationSoundEvent {
                entity,
                sound_handles: sound.sound_handles.clone(),
                speed: 1.0,
                volume: Volume::Linear(ANIMATION_VOLUME),
            });
        }
    }
    Ok(())
}

fn play_animation_on_enter(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    mut query: Query<(Entity, Option<&AnimationSound>)>,
    mut commands: Commands,
) -> Result {
    for (entity, animation) in query.iter_mut() {
        let Some(sound) = &animation else {
            continue;
        };

        let AnimationSoundTrigger::OnEnter = sound.sound_trigger else {
            continue;
        };

        sound_events.write(PlayAnimationSoundEvent {
            entity,
            sound_handles: sound.sound_handles.clone(),
            speed: 1.0,
            volume: Volume::Linear(ANIMATION_VOLUME),
        });

        commands.entity(entity).remove::<AnimationSound>();
    }
    Ok(())
}
