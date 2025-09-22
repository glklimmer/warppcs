use bevy::prelude::*;

use crate::{networking::WorldDirection, server::physics::movement::Velocity};

#[derive(Component)]
#[relationship(relationship_target = ArmySlots)]
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
) {
    for (slot, mut transform) in query.iter_mut() {
        let Ok(army) = commander.get(slot.commander) else {
            continue;
        };

        if let Ok(target_transform) = target.get(slot.commander) {
            let direction: f32 = army.target_direction.into();
            transform.translation.x = target_transform.translation.x + -(slot.offset * direction);
        }
    }
}

fn init_army_direction_timer(
    trigger: Trigger<OnInsert, ArmySlots>,
    mut query: Query<&mut ArmySlots>,
) {
    let Ok(mut army) = query.get_mut(trigger.target()) else {
        return;
    };

    army.opposite_direction_timer = Timer::from_seconds(2., TimerMode::Once);
}

fn army_change_direction(mut query: Query<(&mut ArmySlots, &Velocity)>, time: Res<Time>) {
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
}
