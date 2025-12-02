use bevy::prelude::*;

use chest::open_chest;
use flag::{DropFlagEvent, PickFlagEvent, drop_flag, flag_interact, pick_flag};
use interaction::InteractionTriggeredEvent;
use items::pickup_item;
use mount::MountPlugin;

use crate::server::players::knockout::KnockoutPlugin;

pub mod chest;
pub mod flag;
pub mod interaction;
pub mod items;
pub mod knockout;
pub mod mount;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MountPlugin, KnockoutPlugin))
            .add_event::<DropFlagEvent>()
            .add_event::<PickFlagEvent>()
            .add_systems(
                FixedUpdate,
                (
                    (flag_interact, open_chest, pickup_item)
                        .run_if(on_message::<InteractionTriggeredEvent>),
                    drop_flag.run_if(on_message::<DropFlagEvent>),
                    pick_flag.run_if(on_message::<PickFlagEvent>),
                ),
            );
    }
}
