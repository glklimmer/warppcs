use bevy::prelude::*;

pub struct UIAnimationsPlugin;

#[derive(Component)]
pub struct SpriteShaking {
    timer: Timer,
    intensity: f32,
    original_translation: Vec3,
}

impl SpriteShaking {
    pub fn new(duration_seconds: f32, intensity: f32, original_translation: Vec3) -> Self {
        Self {
            timer: Timer::from_seconds(duration_seconds, TimerMode::Once),
            intensity,
            original_translation,
        }
    }
}

impl Plugin for UIAnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_shake_effect);
    }
}

fn apply_shake_effect(
    mut query: Query<(Entity, &mut Transform, &mut SpriteShaking)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut transform, mut shaking) in query.iter_mut() {
        shaking.timer.tick(time.delta());

        if shaking.timer.is_finished() {
            transform.translation = shaking.original_translation;
            commands.entity(entity).remove::<SpriteShaking>();
        } else {
            let offset_x = fastrand::f32() * 2.0 * shaking.intensity - shaking.intensity; // Range [-intensity, intensity]
            let offset_y = fastrand::f32() * 2.0 * shaking.intensity - shaking.intensity;
            let offset_z = fastrand::f32() * 2.0 * shaking.intensity - shaking.intensity;

            transform.translation =
                shaking.original_translation + Vec3::new(offset_x, offset_y, offset_z);
        }
    }
}
