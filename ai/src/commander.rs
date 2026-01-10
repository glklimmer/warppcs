use bevy::prelude::*;
use bevy_behave::{
    behave,
    prelude::{Behave, BehaveInterrupt, BehaveTree, Tree},
};
use units::{Unit, UnitType};

use crate::{
    Attack, BehaveSources, BehaveTarget, BeingPushed, RetreatToBase, TargetInMeleeRange,
    TargetInProjectileRange, TargetInSightRange, UnitBehaviour,
    movement::{FollowFlag, IsFriendlyFormationUnitInFront},
    retreat::GeneralInSightRange,
};

pub struct AICommanderPlugin;

impl Plugin for AICommanderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_insert_commander_behaviour);
    }
}

fn on_insert_commander_behaviour(
    trigger: On<Insert, UnitBehaviour>,
    units: Query<&Unit>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.entity;
    let unit = units.get(entity)?;

    if unit.unit_type.ne(&UnitType::Commander) {
        return Ok(());
    }

    let king_within_range = behave!(
        Behave::Sequence =>{
            Behave::trigger(GeneralInSightRange),
            Behave::spawn_named("Retreat to Base", (RetreatToBase, BehaveTarget(entity)))
        }
    );
    let mut attack_chain: Vec<Tree<Behave>> = Vec::new();

    attack_chain.push(behave!(
        Behave::Sequence => {
            Behave::trigger(TargetInMeleeRange),
            Behave::spawn_named(
                "Attack nearest enemy Melee",
                (
                    Attack::Melee,
                    BehaveInterrupt::by(TargetInProjectileRange).or_not(TargetInMeleeRange),
                    BehaveTarget(entity),
                ),
            )
        }
    ));

    let tartget_in_sight = behave!(
        Behave::Sequence => {
            Behave::trigger(TargetInSightRange),
            Behave::spawn_named(
                "Commander",
                (
                    BehaveTarget(entity),
                ),
            )
        }
    );

    let tree = behave!(
        Behave::Forever => {
            Behave::Fallback => {
                        @king_within_range,
                        ...attack_chain,
                        @tartget_in_sight,
                        @behave!(
                            Behave::spawn_named(
                                "Following flag",
                                    (FollowFlag, BehaveTarget(entity), BehaveInterrupt::by(TargetInSightRange).or(BeingPushed).or(IsFriendlyFormationUnitInFront))
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
