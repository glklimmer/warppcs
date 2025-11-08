use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};

use crate::{networking::WorldDirection, server::physics::movement::Velocity};

#[derive(Component, Serialize, Deserialize)]
#[relationship(relationship_target = ArmySlots)]
#[require(Replicated, Velocity, Transform)]
pub struct ArmySlot {
    #[relationship]
    pub commander: Entity,
    pub offset: f32,
}

#[derive(Component)]
#[relationship_target(relationship = ArmySlot)]
pub struct ArmySlots {
    #[relationship]
    slots: Vec<Entity>,
    target_direction: WorldDirection,
    opposite_direction_timer: Timer,
}

pub struct ArmySlotPlugin;

impl Plugin for ArmySlotPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_army_direction_timer)
            .add_systems(FixedUpdate, (army_slot_follow, army_change_direction));
    }
}

fn army_slot_follow(
    mut query: Query<(&ArmySlot, &mut Transform)>,
    target: Query<&Transform, Without<ArmySlot>>,
    commander: Query<&ArmySlots>,
) -> Result {
    for (slot, mut transform) in query.iter_mut() {
        let army = commander.get(slot.commander)?;

        let target_transform = target.get(slot.commander)?;
        let direction: f32 = army.target_direction.into();
        transform.translation.x = target_transform.translation.x + -(slot.offset * direction);
    }
    Ok(())
}

fn init_army_direction_timer(
    trigger: Trigger<OnInsert, ArmySlots>,
    mut query: Query<&mut ArmySlots>,
) -> Result {
    let mut army = query.get_mut(trigger.target())?;

    army.opposite_direction_timer = Timer::from_seconds(2., TimerMode::Once);
    Ok(())
}

fn army_change_direction(mut query: Query<(&mut ArmySlots, &Velocity)>, time: Res<Time>) -> Result {
    for (mut army, velocity) in &mut query {
        army.opposite_direction_timer.tick(time.delta());

        let current_direction: WorldDirection = velocity.0.x.into();
        if current_direction.eq(&army.target_direction) || velocity.0.x == 0. {
            army.opposite_direction_timer.reset();
        }

        if army.opposite_direction_timer.just_finished() {
            army.target_direction = current_direction;
        }
    }
    Ok(())
}
