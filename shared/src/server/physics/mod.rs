use std::time::Duration;

use bevy::prelude::*;

use attachment::AttachmentPlugin;
use movement::{MovementPlugin, Velocity};
use projectile::ProjectilePlugin;

use crate::server::physics::army_slot::ArmySlotPlugin;

use super::entities::health::TakeDamage;

pub mod army_slot;
pub mod attachment;
pub mod movement;
pub mod projectile;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MovementPlugin,
            ProjectilePlugin,
            AttachmentPlugin,
            ArmySlotPlugin,
        ))
        .add_systems(FixedUpdate, (apply_force_on_hit, push_back_timer));
    }
}

#[derive(Component)]
pub struct PushBack {
    pub timer: Timer,
}

impl Default for PushBack {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(1., TimerMode::Once);
        timer.tick(Duration::MAX);
        Self { timer }
    }
}
fn apply_force_on_hit(
    mut hit: EventReader<TakeDamage>,
    mut query: Query<(&mut Velocity, &mut PushBack)>,
) {
    for event in hit.read() {
        if let Ok((mut velocity, mut push_back)) = query.get_mut(event.target_entity) {
            if push_back.timer.finished() {
                let direction: f32 = event.direction.into();
                let push = Vec2::new(direction * 50., 50.);
                push_back.timer.reset();
                velocity.0 += push;
            }
        }
    }
}

fn push_back_timer(mut query: Query<&mut PushBack>, time: Res<Time>) {
    for mut push_back in &mut query {
        push_back.timer.tick(time.delta());
    }
}
