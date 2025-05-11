use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::{
    Player, Vec3LayerExt,
    map::Layers,
    networking::LobbyEvent,
    server::{
        physics::movement::Velocity,
        players::items::{Item, ItemType, Rarity},
    },
};

pub struct PlayerLoadout;

impl Plugin for PlayerLoadout {
    fn build(&self, app: &mut App) {
        app.add_event::<GiveRandomItem>();
        app.add_observer(give_random_item);
        app.add_systems(
            Update,
            give_random_items
                .after(ServerSet::Receive)
                .run_if(server_or_singleplayer),
        );
    }
}

fn give_random_items(
    mut lobby_events: EventReader<FromClient<LobbyEvent>>,
    mut commands: Commands,
    players: Query<&Transform, With<Player>>,
) {
    for FromClient {
        client_entity: _,
        event,
    } in lobby_events.read()
    {
        #[allow(irrefutable_let_patterns)]
        let LobbyEvent::StartGame = &event else {
            continue;
        };

        for player_transform in players.iter() {
            for item_type in ItemType::all_variants() {
                let translation = player_transform.translation;
                let item = Item::builder()
                    .with_rarity(Rarity::Common)
                    .with_type(item_type)
                    .build();

                commands.spawn((
                    item.collider(),
                    item,
                    translation.with_y(12.5).with_layer(Layers::Item),
                    Velocity(Vec2::new((fastrand::f32() - 0.5) * 100., 100.)),
                ));
            }
        }
    }
}

#[derive(Event)]
pub struct GiveRandomItem;

fn give_random_item(
    trigger: Trigger<GiveRandomItem>,
    mut commands: Commands,
    players: Query<&Transform, With<Player>>,
) {
    for player_transform in players.iter() {
        for item_type in ItemType::all_variants() {
            let translation = player_transform.translation;
            let item = Item::builder()
                .with_rarity(Rarity::Common)
                .with_type(item_type)
                .build();

            commands.spawn((
                item.collider(),
                item,
                translation.with_y(12.5).with_layer(Layers::Item),
                Velocity(Vec2::new((fastrand::f32() - 0.5) * 100., 100.)),
            ));
        }
    }
}

// #[derive(Event, Deserialize, Serialize)]
// struct LoadoutChoice {
//     item_type: ItemType,
//     choice: Choice,
// }
//
// #[derive(Deserialize, Serialize)]
// enum Choice {
//     Left,
//     Middle,
//     Right,
// }
