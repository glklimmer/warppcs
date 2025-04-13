use bevy::prelude::*;

use chest::open_chest;
use flag::{DropFlagEvent, PickFlagEvent, drop_flag, flag_interact, pick_flag};
use interaction::InteractionTriggeredEvent;
use items::pickup_item;
use mount::mount;

pub mod chest;
pub mod flag;
pub mod interaction;
pub mod items;
pub mod mount;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DropFlagEvent>();
        app.add_event::<PickFlagEvent>();

        app.add_systems(
            FixedUpdate,
            (
                (mount, flag_interact, open_chest, pickup_item)
                    .run_if(on_event::<InteractionTriggeredEvent>),
                drop_flag.run_if(on_event::<DropFlagEvent>),
                pick_flag.run_if(on_event::<PickFlagEvent>),
            ),
        );
    }
}
