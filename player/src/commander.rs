use bevy::prelude::*;

use army::commander::{ActiveCommander, ActiveCommanderExt, CommanderCampInteraction};
use bevy_replicon::prelude::FromClient;
use buildings::siege_camp::SiegeCamp;
use lobby::{ClientPlayerMap, ClientPlayerMapExt};
use shared::{GameSceneId, Owner, Vec3LayerExt, map::Layers};

pub(crate) struct CommanderPlugin;

impl Plugin for CommanderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_camp_interaction);
    }
}

fn handle_camp_interaction(
    trigger: On<FromClient<CommanderCampInteraction>>,
    active: Res<ActiveCommander>,
    client_player_map: ResMut<ClientPlayerMap>,
    query: Query<(&Transform, &GameSceneId)>,
    mut commands: Commands,
) -> Result {
    let player = client_player_map.get_player(&trigger.client_id)?;
    let commander = active.get_entity(player)?;
    let (commander_transform, game_scene_id) = query.get(*commander)?;
    let commander_pos = commander_transform.translation;

    commands.spawn((
        SiegeCamp::default(),
        commander_pos.with_layer(Layers::Building),
        Owner::Player(*player),
        *game_scene_id,
    ));
    Ok(())
}
