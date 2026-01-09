use bevy::prelude::*;

use crate::siege_camp::SiegeCamp;

pub(crate) mod tent;

pub(crate) struct SiegeCampAnimationPlugin;

impl Plugin for SiegeCampAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_camp_sprite);
    }
}

fn init_camp_sprite(
    trigger: On<Add, SiegeCamp>,
    mut camp: Query<&mut Sprite>,
    asset_server: Res<AssetServer>,
) -> Result {
    let mut sprite = camp.get_mut(trigger.entity)?;
    sprite.image = asset_server.load::<Image>("sprites/buildings/siege_camp.png");
    Ok(())
}
