use std::ops::Mul;

use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use shared::{ControlledPlayer, GameState, Owner, server::entities::UnitAnimation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MusicTrack {
    Base,
    Combat,
}

#[derive(Resource)]
struct MusicState {
    current_track: MusicTrack,
    desired_track: MusicTrack,
    is_transitioning: bool,
}

impl Default for MusicState {
    fn default() -> Self {
        MusicState {
            current_track: MusicTrack::Base,
            desired_track: MusicTrack::Base,
            is_transitioning: false,
        }
    }
}

#[derive(Component)]
struct MusicPlayer {
    track: MusicTrack,
    volume: Volume,
    target_volume: Volume,
    fade_speed: FadeUnit,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct FadeUnit(f32);

impl FadeUnit {
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
}

impl From<f32> for FadeUnit {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl Mul<f32> for FadeUnit {
    type Output = f32;

    fn mul(self, rhs: f32) -> Self::Output {
        self.0 * rhs
    }
}

trait VolumeDiff {
    fn diff(&self, other: &Volume) -> f32;
}

impl VolumeDiff for Volume {
    fn diff(&self, other: &Volume) -> f32 {
        (self.to_linear() - other.to_linear()).abs()
    }
}

#[derive(Message)]
struct MusicTransitionEvent {
    track: MusicTrack,
    fade_time: Seconds,
}

#[derive(Deref, PartialEq, PartialOrd)]
struct Seconds(f32);

impl From<f32> for Seconds {
    fn from(val: f32) -> Self {
        Seconds(val)
    }
}

const BACKGROUND_VOLUME: f32 = 0.075;
const COMBAT_DISTANCE_THRESHOLD: f32 = 150.0;

pub struct BackgroundSoundPlugin;

impl Plugin for BackgroundSoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>();
        app.add_event::<MusicTransitionEvent>();

        app.add_systems(Startup, setup_music);
        app.add_systems(
            Update,
            (
                handle_music_transitions.run_if(on_message::<MusicTransitionEvent>),
                update_music_volume,
            ),
        );

        app.add_systems(
            Update,
            play_fight_music.run_if(in_state(GameState::GameSession)),
        );
    }
}

fn setup_music(asset_server: Res<AssetServer>, mut commands: Commands) {
    let menu_track = asset_server.load("music/music_chill.ogg");
    let combat_track = asset_server.load("music/music_fight.ogg");

    let music_tracks = [
        (MusicTrack::Combat, combat_track.clone(), true),
        (MusicTrack::Base, menu_track.clone(), false),
    ];

    commands.spawn_batch(music_tracks.into_iter().map(|(track, handle, paused)| {
        (
            AudioPlayer::new(handle),
            PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::Linear(BACKGROUND_VOLUME),
                paused,
                ..default()
            },
            MusicPlayer {
                track,
                volume: Volume::Linear(BACKGROUND_VOLUME),
                target_volume: Volume::Linear(0.0),
                fade_speed: 1.0.into(),
            },
        )
    }));
}

fn handle_music_transitions(
    mut music_state: ResMut<MusicState>,
    mut music_players: Query<(&mut MusicPlayer, &AudioSink)>,
    mut transition_events: MessageReader<MusicTransitionEvent>,
) {
    for event in transition_events.read() {
        music_state.is_transitioning = true;

        // Set target volumes and fade speeds for all players
        for (mut music_player, sink) in music_players.iter_mut() {
            if music_player.track == event.track {
                music_player.target_volume = Volume::Linear(BACKGROUND_VOLUME);
                sink.play();
            } else {
                music_player.target_volume = Volume::Linear(0.0);
                sink.pause();
            }

            // Calculate fade speed based on desired fade time
            let volume_diff = music_player.target_volume.diff(&music_player.volume);
            if volume_diff > 0.01 && event.fade_time > 0.0.into() {
                music_player.fade_speed = (volume_diff / *event.fade_time).into();
            } else {
                music_player.fade_speed = 1.0.into(); // Default fade speed
            }
        }
    }
}

/// System to update music volume for smooth transitions
fn update_music_volume(
    mut music_state: ResMut<MusicState>,
    mut music_players: Query<(&mut MusicPlayer, &mut AudioSink)>,
    time: Res<Time>,
) {
    if !music_state.is_transitioning {
        return;
    }

    let dt = time.delta_secs();
    let mut all_faded = true;

    for (mut player, mut sink) in music_players.iter_mut() {
        if player.volume.diff(&player.target_volume) > 0.01 {
            let volume = if player.volume.lt(&player.target_volume) {
                (player.volume.to_linear() + player.fade_speed * dt)
                    .min(player.target_volume.to_linear())
            } else {
                (player.volume.to_linear() - player.fade_speed * dt)
                    .max(player.target_volume.to_linear())
            };
            player.volume = Volume::Linear(volume);

            sink.set_volume(player.volume);
            all_faded = false;
        }

        if player.volume.to_linear() >= 0.99 && player.track != music_state.current_track {
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
    mut music_events: MessageWriter<MusicTransitionEvent>,
    player_query: Query<(&Transform, &Owner), With<ControlledPlayer>>,
    unit_query: Query<(&Transform, &Owner, &UnitAnimation), Without<ControlledPlayer>>,
) -> Result {
    let (player_transform, player_owner) = player_query.single()?;
    let mut enemy_nearby = false;
    for (unit_transform, unit_owner, unit_animations) in unit_query.iter() {
        if unit_animations.eq(&UnitAnimation::Death) {
            continue;
        }
        if player_owner.ne(unit_owner) {
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
            music_events.write(MusicTransitionEvent {
                track: MusicTrack::Combat,
                fade_time: 1.5.into(),
            });
        }
    } else if MusicTrack::Base != music_state.desired_track {
        music_state.desired_track = MusicTrack::Base;
        music_events.write(MusicTransitionEvent {
            track: MusicTrack::Base,
            fade_time: 1.5.into(),
        });
    }
    Ok(())
}
