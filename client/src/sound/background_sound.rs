use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use shared::{
    networking::{FactionComparison, Owner},
    GameState,
};

use crate::networking::ControlledPlayer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum MusicTrack {
    #[default]
    None,
    MainMenu,
    Base,
    Combat,
    Victory,
    GameOver,
}

#[derive(Resource, Default)]
struct MusicState {
    current_track: MusicTrack,
    desired_track: MusicTrack,
    is_transitioning: bool,
    volume: f32, // 0.0 to 1.0 scale for dynamic music intensity
}

#[derive(Component)]
struct MusicPlayer {
    track: MusicTrack,
    volume: f32,
    target_volume: f32,
    fade_speed: f32,
}

#[derive(Event)]
struct MusicTransitionEvent {
    track: MusicTrack,
    fade_time: f32, // in seconds
}

#[derive(Event)]
struct AdjustSpatialSoundEvent;

const BACKGROUND_VOLUME: f32 = 0.10;
const COMBAT_DISTANCE_THRESHOLD: f32 = 550.0;
pub struct BackgroundSoundPlugin;

impl Plugin for BackgroundSoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>();
        app.add_event::<MusicTransitionEvent>();
        app.add_event::<AdjustSpatialSoundEvent>();

        app.add_systems(Startup, setup_music);
        app.add_systems(PostUpdate, (handle_music_transitions, update_music_volume));

        app.add_systems(
            PostUpdate,
            (play_fight_music).run_if(in_state(GameState::GameSession)),
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
        volume: 1.0,
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
        Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
        AudioPlayer::<AudioSource>(asset_server.load(path)),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.0),
            paused: true,
            spatial: true,
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
    mut music_players: Query<(&mut MusicPlayer, &SpatialAudioSink)>,
    mut transition_events: EventReader<MusicTransitionEvent>,
) {
    for event in transition_events.read() {
        music_state.is_transitioning = true;

        // Set target volumes and fade speeds for all players
        for (mut music_player, sink) in music_players.iter_mut() {
            if music_player.track == event.track {
                music_player.target_volume = music_state.volume;
                sink.play();
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
    mut music_players: Query<(&mut MusicPlayer, &SpatialAudioSink)>,
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
    player_query: Query<(&Transform, &Owner), With<ControlledPlayer>>,
    unit_query: Query<(&Transform, &Owner), Without<ControlledPlayer>>,
) {
    if let Ok((player_transform, player_owner)) = player_query.get_single() {
        let mut enemy_nearby = false;
        for (unit_transform, unit_owner) in unit_query.iter() {
            if player_owner.is_different_faction(unit_owner) {
                let enemy_distance = player_transform
                    .translation
                    .distance(unit_transform.translation);
                if enemy_distance <= COMBAT_DISTANCE_THRESHOLD {
                    enemy_nearby = true;
                    break;
                }
            }
        }
        if enemy_nearby {
            if MusicTrack::Combat != music_state.desired_track {
                music_state.desired_track = MusicTrack::Combat;
                music_state.volume = BACKGROUND_VOLUME;
                music_events.send(MusicTransitionEvent {
                    track: MusicTrack::Combat,
                    fade_time: 1.5,
                });
            }
        } else {
            if MusicTrack::Base != music_state.desired_track {
                music_state.desired_track = MusicTrack::Base;
                music_state.volume = BACKGROUND_VOLUME;
                music_events.send(MusicTransitionEvent {
                    track: MusicTrack::Base,
                    fade_time: 1.5,
                });
            }
        }
    }
}
