use bevy::prelude::*;
use shared::GameState;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AssetsToLoad(pub Vec<UntypedHandle>);

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetsToLoad>().add_systems(
            Update,
            check_assets_ready.run_if(in_state(GameState::Loading)),
        );
    }
}

fn check_assets_ready(
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    assets_to_load: Res<AssetsToLoad>,
) {
    let all_loaded = assets_to_load
        .iter()
        .all(|handle| asset_server.is_loaded(handle.id()));

    if all_loaded {
        next_state.set(GameState::MainMenu);
    }
}
