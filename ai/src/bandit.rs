use bevy::prelude::*;

use bevy_behave::{
    behave,
    prelude::{Behave, BehaveInterrupt, BehaveTree},
};

use crate::{
    BehaveSources, BehaveTarget, BeingPushed, DetermineTarget, attack_and_walk_in_range,
    movement::Roam,
};

pub struct AIBanditPlugin;

impl Plugin for AIBanditPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_insert_bandit_behaviour);
    }
}

#[derive(Debug, Component, Default, Clone)]
pub enum BanditBehaviour {
    #[default]
    Aggressive,
}

fn on_insert_bandit_behaviour(
    trigger: On<Insert, BanditBehaviour>,
    query: Query<&BanditBehaviour>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.entity;
    let behaviour = query.get(entity)?;
    info!("Bandit behaviour inserted: {:?}", behaviour);
    let stance = match behaviour {
        BanditBehaviour::Aggressive => behave!(Behave::spawn_named(
            "Roaming",
            (
                Roam::default(),
                BehaveInterrupt::by(DetermineTarget).or(BeingPushed),
                BehaveTarget(entity)
            )
        )),
    };

    let attack_chain = attack_and_walk_in_range(entity);

    let tree = behave!(
        Behave::Forever => {
            Behave::Fallback => {
                @ attack_chain,
                @ stance
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
