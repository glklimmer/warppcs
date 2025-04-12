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
    query: Query<&SpatialAudioSink>,
    mut commands: Commands,
) {
    if let Ok(sink) = query.get(trigger.entity()) {
        sink.stop();
        sink.pause();
        commands.entity(trigger.entity()).remove::<AudioPlayer>();
    }
}

fn handle_multiple_animation_sound(
    mut commands: Commands,
    mut sound_events: EventReader<PlayAnimationSoundEvent>,
) {
    for event in sound_events.read() {
        if event.sound_handles.len() < 1 {
            continue;
        }

        let random_sound = fastrand::choice(event.sound_handles.iter()).unwrap();
        if let Some(mut entity_command) = commands.get_entity(event.entity) {
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
}

fn handle_single_animation_sound(
    mut commands: Commands,
    mut sound_events: EventReader<PlayAnimationSoundEvent>,
) {
    for event in sound_events.read() {
        if event.sound_handles.len() != 1 {
            continue;
        };

        if let Some(mut entity_command) = commands.get_entity(event.entity) {
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
    }
}

fn play_sound_on_entity_change(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    mut entity_change_events: EventReader<AnimationChangeEvent>,
    asset_server: Res<AssetServer>,
) {
    for event in entity_change_events.read() {
        let sound = match event.change {
            AnimationChange::Hit(hit_by) => match hit_by {
                Hitby::Arrow => "animation_sound/arrow/arrow_hits_flesh.ogg",
                Hitby::Melee => "animation_sound/arrow/arrow_hits_flesh.ogg",
            },
            AnimationChange::Mount => "animation_sound/horse/horse_sound.ogg",
            _ => continue,
        };

        sound_events.send(PlayAnimationSoundEvent {
            entity: event.entity,
            sound_handles: if sound.is_empty() {
                Vec::new()
            } else {
                vec![asset_server.load(sound)]
            },
            speed: 1.0,
            volume: Volume::new(ANIMATION_VOLUME),
        });
    }
}

fn play_animation_on_projectile_spawn(
    trigger: Trigger<OnAdd, ProjectileType>,
    mut projectile: Query<&ProjectileType>,
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    asset_server: Res<AssetServer>,
) {
    let Ok(projectile_type) = projectile.get_mut(trigger.entity()) else {
        return;
    };

    let sound_handles = match projectile_type {
        ProjectileType::Arrow => vec![asset_server.load("animation_sound/arrow/arrow_flying.ogg")],
    };

    sound_events.send(PlayAnimationSoundEvent {
        entity: trigger.entity(),
        sound_handles,
        speed: 1.5,
        volume: Volume::new(ANIMATION_VOLUME),
    });
}

fn play_on_interactable(
    trigger: Trigger<InteractableSound>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
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
    };
    commands.spawn((
        AudioPlayer::<AudioSource>(audio_file),
        PlaybackSettings {
            mode: PlaybackMode::Remove,
            volume: Volume::new(ANIMATION_VOLUME * 0.5),
            spatial: true,
            ..default()
        },
    ));
}

fn play_animation_on_frame_timer(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    query: Query<(
        Entity,
        &SpriteSheetAnimation,
        Option<&AnimationSound>,
        &Sprite,
    )>,
) {
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
            sound_events.send(PlayAnimationSoundEvent {
                entity,
                sound_handles: sound.sound_handles.clone(),
                speed: 1.0,
                volume: Volume::new(ANIMATION_VOLUME),
            });
        }
    }
}

fn play_animation_on_enter(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    mut query: Query<(Entity, Option<&AnimationSound>)>,
    mut commands: Commands,
) {
    for (entity, animation) in query.iter_mut() {
        let Some(sound) = &animation else {
            continue;
        };

        let AnimationSoundTrigger::OnEnter = sound.sound_trigger else {
            continue;
        };

        sound_events.send(PlayAnimationSoundEvent {
            entity,
            sound_handles: sound.sound_handles.clone(),
            speed: 1.0,
            volume: Volume::new(ANIMATION_VOLUME),
        });

        commands.entity(entity).remove::<AnimationSound>();
    }
}
