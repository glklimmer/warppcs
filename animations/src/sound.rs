use bevy::prelude::*;

use bevy::audio::{PlaybackMode, Volume};
use interaction::{InteractableSound, InteractionType};
use projectiles::ProjectileType;
use shared::{AnimationChange, AnimationChangeEvent, Hitby};

use crate::SpriteSheetAnimation;

pub const CRAFTING_SOUND_PATH: &str = "animation_sound/crafting";
pub const DIRT_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/dirt_footsteps";
pub const GRASS_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/grass_footsteps";
// pub const SNOW_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/snow_footsteps";
// pub const STONE_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/stone_footsteps";
// pub const WATER_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/water_footsteps";
// pub const HORSE_SOUND_PATH: &str = "animation_sound/horse";

#[derive(Message)]
struct PlayAnimationSoundEvent {
    entity: Entity,
    sound_handles: Vec<Handle<AudioSource>>,
    speed: f32,
    volume: Volume,
}

const ANIMATION_VOLUME: f32 = 0.25;

#[derive(Component, Clone, Default, PartialEq, Eq)]
pub enum AnimationSoundTrigger {
    #[default]
    Enter,
    StartFrameTimer,
    EndFrameTimer,
}

#[derive(Component, Clone)]
#[require(AnimationSoundTrigger)]
pub struct AnimationSound {
    pub sound_handles: Vec<Handle<AudioSource>>,
    pub sound_trigger: AnimationSoundTrigger,
}

pub struct AnimationSoundPlugin;

impl Plugin for AnimationSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayAnimationSoundEvent>();
        app.add_observer(play_animation_on_projectile_spawn);
        app.add_observer(play_on_interactable);
        app.add_observer(stop_animation_sound_on_remove);

        app.add_systems(
            Update,
            (
                handle_single_animation_sound.run_if(on_message::<PlayAnimationSoundEvent>),
                handle_multiple_animation_sound.run_if(on_message::<PlayAnimationSoundEvent>),
                play_sound_on_entity_change,
                play_animation_on_frame_timer,
                play_animation_on_enter,
            ),
        );
    }
}

fn stop_animation_sound_on_remove(
    trigger: On<Remove, AnimationSound>,
    query: Query<Option<&SpatialAudioSink>>,
    mut commands: Commands,
) -> Result {
    let maybe_sink = query.get(trigger.entity)?;
    if let Some(sink) = maybe_sink {
        sink.stop();
        sink.pause();
    };
    commands.entity(trigger.entity).remove::<AudioPlayer>();
    Ok(())
}

fn handle_multiple_animation_sound(
    mut sound_events: MessageReader<PlayAnimationSoundEvent>,
    mut commands: Commands,
) -> Result {
    for event in sound_events.read() {
        let Some(random_sound) = fastrand::choice(event.sound_handles.iter()) else {
            continue;
        };
        if let Ok(mut entity_command) = commands.get_entity(event.entity) {
            entity_command.insert((
                AudioPlayer::<AudioSource>(random_sound.clone()),
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
    mut sound_events: MessageReader<PlayAnimationSoundEvent>,
    mut commands: Commands,
) -> Result {
    for event in sound_events.read() {
        if event.sound_handles.len() != 1 {
            continue;
        };

        let mut entity_command = commands.get_entity(event.entity)?;
        entity_command.insert((
            AudioPlayer::<AudioSource>(event.sound_handles[0].clone()),
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
    mut sound_events: MessageWriter<PlayAnimationSoundEvent>,
    mut entity_change_events: MessageReader<AnimationChangeEvent>,
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
}

fn play_animation_on_projectile_spawn(
    trigger: On<Add, ProjectileType>,
    mut projectile: Query<&ProjectileType>,
    mut sound_events: MessageWriter<PlayAnimationSoundEvent>,
    asset_server: Res<AssetServer>,
) -> Result {
    let projectile_type = projectile.get_mut(trigger.entity)?;

    let sound_handles = match projectile_type {
        ProjectileType::Arrow => vec![asset_server.load("animation_sound/arrow/arrow_flying.ogg")],
    };

    sound_events.write(PlayAnimationSoundEvent {
        entity: trigger.entity,
        sound_handles,
        speed: 1.5,
        volume: Volume::Linear(ANIMATION_VOLUME),
    });
    Ok(())
}

fn play_on_interactable(
    trigger: On<InteractableSound>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
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
}

fn play_animation_on_frame_timer(
    mut sound_events: MessageWriter<PlayAnimationSoundEvent>,
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
        let AnimationSoundTrigger::StartFrameTimer = sound.sound_trigger else {
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
}

fn play_animation_on_enter(
    mut sound_events: MessageWriter<PlayAnimationSoundEvent>,
    mut query: Query<(Entity, Option<&AnimationSound>)>,
    mut commands: Commands,
) {
    for (entity, animation) in query.iter_mut() {
        let Some(sound) = &animation else {
            continue;
        };

        let AnimationSoundTrigger::Enter = sound.sound_trigger else {
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
}
