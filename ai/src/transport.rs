use bevy::prelude::*;

use bevy_behave::{Behave, behave, prelude::BehaveTree};
use transport::Transport;

use crate::{BehaveSources, BehaveTarget, TravelToEntity};

pub(crate) struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_spawn_transport);
    }
}

fn on_spawn_transport(
    spawn: On<Add, Transport>,
    query: Query<&Transport>,
    mut commands: Commands,
) -> Result {
    let entity = spawn.entity;
    let target = query.get(entity)?;

    let tree = behave!(
        Behave::Forever => {
            Behave::Fallback => {
                Behave::spawn_named(
                    "Traveling to entity",
                    (
                        TravelToEntity(target.target),
                        BehaveTarget(entity)
                    )
                )
            }
        }
    );

    commands
        .entity(entity)
        .despawn_related::<BehaveSources>()
        .with_child((
            BehaveTree::new(tree).with_logging(false),
            BehaveTarget(entity),
        ));
    Ok(())
}
