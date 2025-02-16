use bevy::prelude::*;
use warppcs::{GameTextures, Winsize, PLAYER_SPRITE}; // Import the PlayerPlugin

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .add_plugins(HelloPlugin)
        .add_systems(Startup, setup_system)
        .run();
}

fn setup_system(
    mut commands: Commands,
    windows: Query<&mut Window>,
    asset_server: Res<AssetServer>,
) {
    let window = windows.get_single().unwrap();

    let win_size = Winsize {
        w: window.width(),
        h: window.height(),
    };
    commands.insert_resource(win_size);

    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
    };
    commands.insert_resource(game_textures);

    commands.spawn(Camera2dBundle::default());
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("Hello, {}!", name.0);
        }
    }
}

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Chicho".to_string())));
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Chicho" {
            name.0 = "Chicho the Great".to_string();
            break;
        }
    }
}

#[derive(Resource)]
struct GreetTimer(Timer);

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_people);
        app.add_systems(Update, (update_people, greet_people).chain());
    }
}

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct Person;
