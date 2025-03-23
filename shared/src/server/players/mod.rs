use bevy::prelude::*;

use flag::{drop_flag, flag_interact, pick_flag, DropFlagEvent, PickFlagEvent};
use interaction::{InteractPlugin, InteractionTriggeredEvent};
use mount::mount;

pub mod flag;
pub mod interaction;
pub mod mount;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InteractPlugin);

        app.add_event::<DropFlagEvent>();
        app.add_event::<PickFlagEvent>();

        app.add_systems(
            FixedUpdate,
            (
                (mount, flag_interact).run_if(on_event::<InteractionTriggeredEvent>),
                drop_flag.run_if(on_event::<DropFlagEvent>),
                pick_flag.run_if(on_event::<PickFlagEvent>),
            ),
        );
    }
}
