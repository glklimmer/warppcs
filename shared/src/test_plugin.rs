use bevy::prelude::*;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};

pub struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_event::<TestEvent>(ChannelKind::Ordered)
            .add_systems(PostUpdate, send_test.before(ClientSet::Send))
            .add_systems(
                PreUpdate,
                recieve_test
                    .after(ServerSet::Receive)
                    .run_if(server_or_singleplayer),
            );
    }
}

#[derive(Debug, Deserialize, Event, Serialize)]
struct TestEvent;

fn send_test(keyboard_input: Res<ButtonInput<KeyCode>>, mut lobby_events: EventWriter<TestEvent>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        println!("sending space");
        lobby_events.send(TestEvent);
    }
}

fn recieve_test(mut lobby_events: EventReader<FromClient<TestEvent>>) {
    for FromClient { client_id, event } in lobby_events.read() {
        info!("received event {event:?} from {client_id:?}");
        println!("------ TEST EVENT ------");
    }
}
