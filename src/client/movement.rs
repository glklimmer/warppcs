use bevy::prelude::*;

use crate::{
    server::movement::Velocity,
    shared::networking::{Movement, UnitType},
};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (set_transform, random_target, move_towards_target));
    }
}

fn set_transform(mut query: Query<(&mut Transform, &Movement)>) {
    for (mut transform, movement) in &mut query {
        transform.translation = movement.translation.into();
    }
}

#[derive(Component)]
struct MovementTarget(Vec3);

#[allow(clippy::type_complexity)]
fn random_target(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&MovementTarget>), (With<UnitType>, With<Movement>)>,
) {
    for (entity, transform, target_option) in query.iter() {
        if target_option.is_none() {
            let mut new_target = transform.translation;
            new_target.x += fastrand::f32();

            commands.entity(entity).insert(MovementTarget(new_target));
        }
    }
}

fn move_towards_target(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut Velocity, &Transform, &MovementTarget),
        (With<UnitType>, With<Movement>),
    >,
) {
    for (entity, velocity, transform, target) in &mut query {
        match transform.translation.x.total_cmp(target.0.x) {
            std::cmp::Ordering::Less => todo!(),
            std::cmp::Ordering::Equal => todo!(),
            std::cmp::Ordering::Greater => {
                velocity.0.x = 0.;
                commands.entity(entity).remove::<MovementTarget>();
            }
        }
    }
}
