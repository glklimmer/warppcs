use bevy::prelude::*;
use shared::GameState;

use crate::entities::player::ClientPlayer;

#[derive(Component)]
pub struct Playing;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameSession), play_music);
        app.add_systems(
            Update,
            play_fight_music.run_if(in_state(GameState::GameSession)),
        );
    }
}

fn play_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(AudioPlayer::<AudioSource>(
        (asset_server.load("music/music_chill.ogg")),
        Playing,
    ));
}

fn play_fight_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    players: Query<&ClientPlayer>,
    music: Query<&AudioPlayer, Without<Playing>>,
    music_box_query: Query<&AudioSink, With<MusicBox>>,
) {
    let count = players.iter().count();
    match music.get_single() {
        Ok(_) => {
            if count >= 2 {
                commands.spawn((
                    AudioPlayer::<AudioSource>(asset_server.load("music/music_fight.ogg")),
                    Playing,
                ));
                println!("playing");
            }
        }
        Err(_) => todo!(),
    }
}
