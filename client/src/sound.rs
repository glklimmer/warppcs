use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use shared::{
    networking::{Hitby, ServerMessages, SpawnProjectile},
    GameState,
};

use crate::{
    animations::{AnimationSoundTrigger, SpriteSheetAnimation},
    entities::player::ClientPlayer,
    networking::NetworkEvent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnimationSound {
    #[default]
    None,
    ArrowHit,
    ArrowFly,
    MeleeHit,
    FootStep,
}

#[derive(Component)]
pub struct AnimationSoundPlayer {
    pub sound_volume: AnimationSound,
    pub volume: f32,
}

#[derive(Event)]
pub struct PlayAnimationSoundEvent {
    pub sound: String,
    pub speed: f32,
    pub volume: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MusicTrack {
    #[default]
    None,
    MainMenu,
    Base,
    Combat,
    Victory,
    GameOver,
}

#[derive(Resource, Default)]
pub struct MusicState {
    pub current_track: MusicTrack,
    pub desired_track: MusicTrack,
    pub is_transitioning: bool,
    pub volume: f32, // 0.0 to 1.0 scale for dynamic music intensity
}

#[derive(Component)]
pub struct MusicPlayer {
    pub track: MusicTrack,
    pub volume: f32,
    pub target_volume: f32,
    pub fade_speed: f32,
}

#[derive(Event)]
pub struct MusicTransitionEvent {
    pub track: MusicTrack,
    pub fade_time: f32, // in seconds
}

const BACKGROUND_VOLUME: f32 = 0.15;
const ANIMATION_VOLUME: f32 = 0.55;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>();
        app.add_event::<MusicTransitionEvent>();
        app.add_event::<PlayAnimationSoundEvent>();

        app.add_systems(Startup, setup_music);
        app.add_systems(Update, (handle_music_transitions, update_music_volume));

        app.add_systems(
            Update,
            (
                play_fight_music,
                handle_animation_sounds,
                play_animation_on_hit.run_if(on_event::<NetworkEvent>),
                play_animation_on_projectile.run_if(on_event::<SpawnProjectile>),
                play_animation_on_frame_timer,
                play_animation_on_enter_leave,
            )
                .run_if(in_state(GameState::GameSession)),
        );
    }
}

fn setup_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_music_track(
        &mut commands,
        &asset_server,
        MusicTrack::MainMenu,
        "music/music_chill.ogg",
    );
    spawn_music_track(
        &mut commands,
        &asset_server,
        MusicTrack::Combat,
        "music/music_fight.ogg",
    );
    spawn_music_track(
        &mut commands,
        &asset_server,
        MusicTrack::Base,
        "music/music_chill.ogg",
    );
    spawn_music_track(
        &mut commands,
        &asset_server,
        MusicTrack::Victory,
        "music/victory.ogg",
    );
    spawn_music_track(
        &mut commands,
        &asset_server,
        MusicTrack::GameOver,
        "music/game_over.ogg",
    );

    commands.insert_resource(MusicState {
        current_track: MusicTrack::None,
        desired_track: MusicTrack::MainMenu,
        is_transitioning: false,
        volume: 0.0,
    });

    commands.send_event(MusicTransitionEvent {
        track: MusicTrack::MainMenu,
        fade_time: 2.0,
    });
}

fn spawn_music_track(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    track: MusicTrack,
    path: &str,
) {
    commands.spawn((
        AudioPlayer::<AudioSource>(asset_server.load(path)),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::new(0.0),
            paused: true,
            ..default()
        },
        MusicPlayer {
            track,
            volume: 0.0,
            target_volume: 0.0,
            fade_speed: 1.0,
        },
    ));
}

fn handle_music_transitions(
    mut music_state: ResMut<MusicState>,
    mut music_players: Query<(&mut MusicPlayer, &AudioSink)>,
    mut transition_events: EventReader<MusicTransitionEvent>,
) {
    for event in transition_events.read() {
        music_state.is_transitioning = true;

        // Set target volumes and fade speeds for all players
        for (mut music_player, sink) in music_players.iter_mut() {
            sink.play();
            if music_player.track == event.track {
                music_player.target_volume = music_state.volume;
            } else {
                music_player.target_volume = 0.0;
            }

            // Calculate fade speed based on desired fade time
            let volume_diff = (music_player.target_volume - music_player.volume).abs();
            if volume_diff > 0.01 && event.fade_time > 0.0 {
                music_player.fade_speed = volume_diff / event.fade_time;
            } else {
                music_player.fade_speed = 1.0; // Default fade speed
            }
        }
    }
}

// System to update music volume for smooth transitions
fn update_music_volume(
    time: Res<Time>,
    mut music_state: ResMut<MusicState>,
    mut music_players: Query<(&mut MusicPlayer, &AudioSink)>,
) {
    if !music_state.is_transitioning {
        return;
    }

    let dt = time.delta_secs();
    let mut all_faded = true;

    for (mut player, sink) in music_players.iter_mut() {
        if (player.volume - player.target_volume).abs() > 0.01 {
            player.volume = if player.volume < player.target_volume {
                (player.volume + player.fade_speed * dt).min(player.target_volume)
            } else {
                (player.volume - player.fade_speed * dt).max(player.target_volume)
            };

            sink.set_volume(player.volume);
            all_faded = false;
        }

        if player.volume >= 0.99 && player.track != music_state.current_track {
            music_state.current_track = player.track;
        }
    }

    // Only mark transition as complete when all volumes have reached their targets
    if all_faded {
        music_state.is_transitioning = false;
    }
}

fn play_fight_music(
    mut music_state: ResMut<MusicState>,
    mut music_events: EventWriter<MusicTransitionEvent>,
    players: Query<(&ClientPlayer, &Transform)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();
    let (track, intensity, fade_time) = (MusicTrack::Combat, BACKGROUND_VOLUME, 1.5);
    let mut visible_players = 0;

    for (_player, transform) in players.iter() {
        if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, transform.translation) {
            if screen_pos.x >= 0.0
                && screen_pos.x <= window.width()
                && screen_pos.y >= 0.0
                && screen_pos.y <= window.height()
            {
                visible_players += 1;
            }
        }
    }

    if visible_players >= 2 {
        if track != music_state.desired_track {
            music_state.desired_track = track;
            music_state.volume = intensity;

            music_events.send(MusicTransitionEvent { track, fade_time });
        }
    } else {
        let (track, intensity, fade_time) = (MusicTrack::Base, BACKGROUND_VOLUME, 1.5);
        music_state.desired_track = track;
        music_state.volume = intensity;

        music_events.send(MusicTransitionEvent { track, fade_time });
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
                mode: PlaybackMode::Once,
                speed: event.speed,
                volume: Volume::new(event.volume),
                ..default()
            },
        ));
    }
}

fn play_animation_on_hit(
    mut sound_events: EventWriter<PlayAnimationSoundEvent>,
    mut network_events: EventReader<NetworkEvent>,
) {
    for event in network_events.read() {
        if let ServerMessages::EntityHit { entity: _, by } = event.message {
            let sound = match by {
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

fn play_animation_on_projectile(
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

        animation.animation_sound = None
    }
}
