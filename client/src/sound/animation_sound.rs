use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use shared::{
    networking::{Hitby, SpawnProjectile},
    GameState,
};

use crate::animations::{AnimationSoundTrigger, Change, EntityChangeEvent, SpriteSheetAnimation};

#[derive(Event)]
struct PlayAnimationSoundEvent {
    sound: String,
    speed: f32,
    volume: f32,
}

const ANIMATION_VOLUME: f32 = 0.35;

pub struct AnimationSoundPlugin;

impl Plugin for AnimationSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayAnimationSoundEvent>();

        app.add_systems(
            Update,
            (
                handle_animation_sounds,
                play_sound_on_entity_change.run_if(on_event::<EntityChangeEvent>),
                play_animation_on_projectile_spawn.run_if(on_event::<SpawnProjectile>),
                play_animation_on_frame_timer,
                play_animation_on_enter_leave,
            )
                .run_if(in_state(GameState::GameSession)),
        );
    }
}

fn handle_animation_sounds(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sound_events: EventReader<PlayAnimationSoundEvent>,
) {
    for event in sound_events.read() {
        commands.spawn((
            AudioPlayer::<AudioSource>(asset_server.load(&event.sound)),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                speed: event.speed,
                volume: Volume::new(event.volume),
                ..default()
            },
        ));
    }
}

fn play_sound_on_entity_change(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    mut entity_change_events: EventReader<EntityChangeEvent>,
) {
    for event in entity_change_events.read() {
        if let Change::Hit(hit_by) = &event.change {
            let sound = match hit_by {
                Hitby::Arrow => "animation_sound/arrow/arrow-hits-flesh.ogg",
                Hitby::Meele => "animation_sound/arrow/arrow-hits-flesh.ogg",
            };
            sound_events.send(PlayAnimationSoundEvent {
                sound: sound.to_string(),
                speed: 1.5,
                volume: ANIMATION_VOLUME,
            });
        }
    }
}

fn play_animation_on_projectile_spawn(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    mut spawn_projectile: EventReader<SpawnProjectile>,
) {
    for _ in spawn_projectile.read() {
        sound_events.send(PlayAnimationSoundEvent {
            sound: "animation_sound/arrow/arrow_flying.ogg".to_string(),
            speed: 1.5,
            volume: ANIMATION_VOLUME,
        });
    }
}

fn play_animation_on_frame_timer(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    query: Query<(&SpriteSheetAnimation, &Sprite)>,
) {
    for (animation, sprite) in query.iter() {
        let sound = match &animation.animation_sound {
            Some(sound) => sound,
            None => continue,
        };

        if sound
            .sound_trigger
            .ne(&AnimationSoundTrigger::OnStartFrameTimer)
        {
            continue;
        }

        let atlas = sprite.texture_atlas.as_ref().unwrap();
        if animation.frame_timer.just_finished() && atlas.index == animation.first_sprite_index {
            sound_events.send(PlayAnimationSoundEvent {
                sound: sound.sound_file.clone(),
                speed: 1.5,
                volume: ANIMATION_VOLUME,
            });
        }
    }
}

fn play_animation_on_enter_leave(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    mut query: Query<&mut SpriteSheetAnimation>,
) {
    for mut animation in query.iter_mut() {
        let sound = match &animation.animation_sound {
            Some(sound) => sound,
            None => continue,
        };

        if sound.sound_trigger.ne(&AnimationSoundTrigger::OnEnter) {
            continue;
        }

        sound_events.send(PlayAnimationSoundEvent {
            sound: sound.sound_file.clone(),
            speed: 1.5,
            volume: ANIMATION_VOLUME,
        });

        animation.animation_sound = None;
    }
}
