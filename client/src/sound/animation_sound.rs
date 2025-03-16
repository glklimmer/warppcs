use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use shared::{
    networking::{Hitby, SpawnProjectile},
    GameState,
};

use crate::{
    animations::{
        units::Unit, AnimationSoundTrigger, Change, EntityChangeEvent, SpriteSheetAnimation,
    },
    networking::NetworkMapping,
};

#[derive(Event)]
struct PlayAnimationSoundEvent {
    entity: Entity,
    sound_files: Vec<String>,
    speed: f32,
    volume: f32,
}

const ANIMATION_VOLUME: f32 = 0.25;

pub struct AnimationSoundPlugin;

impl Plugin for AnimationSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayAnimationSoundEvent>();

        app.add_systems(
            Update,
            (
                handle_single_animation_sound.run_if(on_event::<PlayAnimationSoundEvent>),
                handle_multiple_animation_sound.run_if(on_event::<PlayAnimationSoundEvent>),
                play_sound_on_entity_change.run_if(on_event::<EntityChangeEvent>),
                play_animation_on_projectile_spawn.run_if(on_event::<SpawnProjectile>),
                play_animation_on_frame_timer,
                play_animation_on_enter,
                play_recruite_unit_call,
            )
                .run_if(in_state(GameState::GameSession)),
        );
    }
}

fn handle_multiple_animation_sound(
    mut commands: Commands,
    mut sound_events: EventReader<PlayAnimationSoundEvent>,
    asset_server: ResMut<AssetServer>,
) {
    for event in sound_events.read() {
        if event.sound_files.len() < 1 {
            continue;
        };

        let random_sound = fastrand::choice(event.sound_files.iter()).unwrap();
        if let Some(mut entity_command) = commands.get_entity(event.entity) {
            entity_command.insert((
                AudioPlayer::<AudioSource>(asset_server.load(random_sound)),
                PlaybackSettings {
                    mode: PlaybackMode::Remove,
                    speed: event.speed,
                    volume: Volume::new(event.volume),
                    spatial: true,
                    ..default()
                },
            ));
        }
    }
}

fn handle_single_animation_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sound_events: EventReader<PlayAnimationSoundEvent>,
) {
    for event in sound_events.read() {
        if &event.sound_files.len() > &1 {
            continue;
        };

        if let Some(mut entity_command) = commands.get_entity(event.entity) {
            entity_command.insert((
                AudioPlayer::<AudioSource>(asset_server.load(&event.sound_files[0])),
                PlaybackSettings {
                    mode: PlaybackMode::Remove,
                    speed: event.speed,
                    volume: Volume::new(event.volume),
                    spatial: true,
                    ..default()
                },
            ));
        }
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
                entity: event.entity,
                sound_files: vec![sound.to_string()],
                speed: 1.5,
                volume: ANIMATION_VOLUME,
            });
        }
    }
}

fn play_animation_on_projectile_spawn(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    mut spawn_projectile: EventReader<SpawnProjectile>,
    network_mapping: Res<NetworkMapping>,
) {
    for spawn in spawn_projectile.read() {
        let entity = network_mapping.0.get(&spawn.entity);
        sound_events.send(PlayAnimationSoundEvent {
            entity: *entity.unwrap(),
            sound_files: vec!["animation_sound/arrow/arrow_flying.ogg".to_string()],
            speed: 1.5,
            volume: ANIMATION_VOLUME,
        });
    }
}

fn play_recruite_unit_call(
    mut commands: Commands,
    unit_query: Query<Entity, Added<Unit>>,
    asset_server: Res<AssetServer>,
) {
    for _ in &unit_query {
        commands.spawn((
            AudioPlayer::<AudioSource>(
                asset_server.load("animation_sound/recruitment/recruite_call.ogg".to_string()),
            ),
            PlaybackSettings {
                mode: PlaybackMode::Remove,
                volume: Volume::new(ANIMATION_VOLUME),
                spatial: false,
                ..default()
            },
        ));
    }
}

fn play_animation_on_frame_timer(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    query: Query<(Entity, &SpriteSheetAnimation, &Sprite)>,
) {
    for (entity, animation, sprite) in query.iter() {
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
                entity,
                sound_files: sound.sound_files.clone(),
                speed: 2.0,
                volume: ANIMATION_VOLUME,
            });
        }
    }
}

fn play_animation_on_enter(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    mut query: Query<(Entity, &mut SpriteSheetAnimation)>,
) {
    for (entity, mut animation) in query.iter_mut() {
        let sound = match &animation.animation_sound {
            Some(sound) => sound,
            None => continue,
        };

        if sound.sound_trigger.ne(&AnimationSoundTrigger::OnEnter) {
            continue;
        }

        sound_events.send(PlayAnimationSoundEvent {
            entity,
            sound_files: sound.sound_files.clone(),
            speed: 1.5,
            volume: ANIMATION_VOLUME,
        });

        animation.animation_sound = None;
    }
}
