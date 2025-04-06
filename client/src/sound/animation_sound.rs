use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use shared::{
    server::{entities::Unit, physics::projectile::ProjectileType},
    AnimationChange, AnimationChangeEvent, Hitby, Owner,
};

use crate::{
    animations::{AnimationSound, AnimationSoundTrigger, SpriteSheetAnimation},
    networking::ControlledPlayer,
};

#[derive(Component)]
pub struct CancelAnimationSound;

#[derive(Event)]
struct PlayAnimationSoundEvent {
    entity: Entity,
    sound_files: Vec<String>,
    speed: f32,
    volume: Volume,
}

const ANIMATION_VOLUME: f32 = 0.25;

pub struct AnimationSoundPlugin;

impl Plugin for AnimationSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayAnimationSoundEvent>();
        app.add_observer(play_animation_on_projectile_spawn);
        app.add_observer(play_recruite_unit_call);

        app.add_systems(
            Update,
            (
                handle_single_animation_sound.run_if(on_event::<PlayAnimationSoundEvent>),
                handle_multiple_animation_sound.run_if(on_event::<PlayAnimationSoundEvent>),
                play_sound_on_entity_change,
                play_animation_on_frame_timer,
                play_animation_on_enter,
                stop_sound_cancel,
            ),
        );
    }
}

fn stop_sound_cancel(
    mut commands: Commands,
    query: Query<(Entity, &SpatialAudioSink, Option<&CancelAnimationSound>)>,
) {
    for (entity, sink, cancel) in query.iter() {
        match cancel {
            Some(_) => {
                sink.stop();
                sink.pause();
                commands.entity(entity).remove::<AnimationSound>();
                commands.entity(entity).remove::<CancelAnimationSound>();
                commands.entity(entity).remove::<AudioPlayer>();
            }
            None => return,
        }
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
        }

        let random_sound = fastrand::choice(event.sound_files.iter()).unwrap();
        if let Some(mut entity_command) = commands.get_entity(event.entity) {
            entity_command.insert((
                AudioPlayer::<AudioSource>(asset_server.load(random_sound)),
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
    asset_server: Res<AssetServer>,
    mut sound_events: EventReader<PlayAnimationSoundEvent>,
) {
    for event in sound_events.read() {
        if event.sound_files.len() != 1 {
            continue;
        };

        if let Some(mut entity_command) = commands.get_entity(event.entity) {
            entity_command.insert((
                AudioPlayer::<AudioSource>(asset_server.load(&event.sound_files[0])),
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
) {
    for event in entity_change_events.read() {
        let sound = match event.change {
            AnimationChange::Hit(hit_by) => match hit_by {
                Hitby::Arrow => "animation_sound/arrow/arrow_hits_flesh.ogg",
                Hitby::Melee => "animation_sound/arrow/arrow_hits_flesh.ogg",
            },
            _ => "",
        };
        sound_events.send(PlayAnimationSoundEvent {
            entity: event.entity,
            sound_files: vec![sound.to_string()],
            speed: 1.5,
            volume: Volume::new(ANIMATION_VOLUME),
        });
    }
}

fn play_animation_on_projectile_spawn(
    trigger: Trigger<OnAdd, ProjectileType>,
    mut projectile: Query<&ProjectileType>,
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
) {
    let Ok(projectile_type) = projectile.get_mut(trigger.entity()) else {
        return;
    };

    let sound_files = match projectile_type {
        ProjectileType::Arrow => vec!["animation_sound/arrow/arrow_flying.ogg".to_string()],
    };

    sound_events.send(PlayAnimationSoundEvent {
        entity: trigger.entity(),
        sound_files,
        speed: 1.5,
        volume: Volume::new(ANIMATION_VOLUME),
    });
}

fn play_recruite_unit_call(
    trigger: Trigger<OnAdd, Unit>,
    units: Query<&Owner, With<Unit>>,
    player: Query<&Owner, With<ControlledPlayer>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok(player_owner) = player.get_single() else {
        return;
    };

    let Ok(units_owner) = units.get(trigger.entity()) else {
        return;
    };

    if units_owner.is_different_faction(player_owner) {
        return;
    }

    commands.spawn((
        AudioPlayer::<AudioSource>(
            asset_server.load("animation_sound/recruitment/recruite_call.ogg".to_string()),
        ),
        PlaybackSettings {
            mode: PlaybackMode::Remove,
            volume: Volume::new(ANIMATION_VOLUME * 0.5),
            spatial: false,
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
        let sound = match &animation {
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
        if sprite_animation.frame_timer.just_finished()
            && atlas.index == sprite_animation.first_sprite_index
        {
            sound_events.send(PlayAnimationSoundEvent {
                entity,
                sound_files: sound.sound_files.clone(),
                speed: 2.0,
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
        let sound = match &animation {
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
            volume: Volume::new(ANIMATION_VOLUME),
        });

        commands.entity(entity).remove::<AnimationSound>();
    }
}
