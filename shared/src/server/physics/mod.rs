use bevy::prelude::*;

use attachment::AttachmentPlugin;
use movement::{MovementPlugin, Velocity};
use projectile::ProjectilePlugin;

use crate::networking::Facing;

use super::entities::health::TakeDamage;

pub mod attachment;
pub mod movement;
pub mod projectile;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MovementPlugin);
        app.add_plugins(ProjectilePlugin);
        app.add_plugins(AttachmentPlugin);

        app.add_systems(FixedUpdate, (apply_force_on_hit, push_back_timer));
    }
}

#[derive(Component)]
pub struct PushBack {
    pub timer: Timer,
}

impl Default for PushBack {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1., TimerMode::Once),
        }
    }
}
fn apply_force_on_hit(
    mut hit: EventReader<TakeDamage>,
    mut query: Query<(&mut Velocity, &mut PushBack)>,
) {
    for event in hit.read() {
        if let Ok((mut velocity, mut push_back)) = query.get_mut(event.target_entity) {
            if push_back.timer.finished() {
                let push = match event.direction {
                    Facing::Left => Vec2::new(150., 150.),
                    Facing::Right => Vec2::new(-150., 150.),
                };
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
