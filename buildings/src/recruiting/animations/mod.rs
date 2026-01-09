use bevy::prelude::*;

use crate::{Building, recruiting::RecruitBuilding};

pub(crate) mod archer;
pub(crate) mod pikeman;
pub(crate) mod shieldwarrior;

pub(crate) struct RecruitAnimationPlugin;

impl Plugin for RecruitAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_recruit_building_sprite);
    }
}

fn init_recruit_building_sprite(
    trigger: On<Add, RecruitBuilding>,
    mut slots: Query<&mut Sprite>,
    asset_server: Res<AssetServer>,
) -> Result {
    let mut sprite = slots.get_mut(trigger.entity)?;
    sprite.image = asset_server.load::<Image>(Building::marker_texture());
    Ok(())
}
