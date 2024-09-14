use bevy::prelude::*;

use super::UnitBehaviour;
use crate::shared::networking::{Unit, UnitType};

pub struct AttackPlugin;

#[derive(Event)]
struct TakeDamage {
    target_entity: Entity,
    damage: f32,
}

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamage>();

        app.add_systems(Update, (process_attacks, apply_damage));
    }
}

pub fn unit_range(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 50.,
        UnitType::Pikeman => 140.,
        UnitType::Archer => 600.,
    }
}

pub fn unit_damage(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 12.,
        UnitType::Pikeman => 18.,
        UnitType::Archer => 6.,
    }
}

pub fn unit_health(unit_type: &UnitType) -> f32 {
    match unit_type {
        UnitType::Shieldwarrior => 120.,
        UnitType::Pikeman => 90.,
        UnitType::Archer => 60.,
    }
}

pub fn unit_swing_timer(unit_type: &UnitType) -> Timer {
    let time = match unit_type {
        UnitType::Shieldwarrior => 1.,
        UnitType::Pikeman => 2.,
        UnitType::Archer => 4.,
    };
    Timer::from_seconds(time, TimerMode::Repeating)
}

fn process_attacks(
    mut query: Query<(&UnitBehaviour, &mut Unit)>,
    mut attack_events: EventWriter<TakeDamage>,
    time: Res<Time>,
) {
    for (behaviour, mut unit) in query.iter_mut() {
        if let UnitBehaviour::AttackTarget(target_entity) = behaviour {
            unit.swing_timer.tick(time.delta());
            if unit.swing_timer.finished() {
                println!("Swinging at target: {}", target_entity);
                attack_events.send(TakeDamage {
                    target_entity: *target_entity,
                    damage: unit_damage(&unit.unit_type),
                });
            }
        }
    }
}

fn apply_damage(mut query: Query<&mut Unit>, mut attack_events: EventReader<TakeDamage>) {
    for event in attack_events.read() {
        if let Ok(mut unit) = query.get_mut(event.target_entity) {
            unit.health -= event.damage;
            println!("New health: {}.", unit.health);
        }
    }
}
