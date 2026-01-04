use bevy::prelude::*;

use bevy_replicon::prelude::ToClients;
use health::{Health, TakeDamage};
use shared::{AnimationChangeEvent, Owner};

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, on_unit_death);
    }
}

fn on_unit_death(
    mut damage_events: MessageReader<TakeDamage>,
    mut unit_animation: MessageWriter<ToClients<AnimationChangeEvent>>,
    units: Query<
        (
            Entity,
            &Health,
            &Owner,
            Option<&TargetedBy>,
            Option<&FlagAssignment>,
            Option<&ArmyFlagAssignments>,
        ),
        With<Unit>,
    >,
    group: Query<&FlagUnits>,
    transform: Query<&Transform>,
    holder: Query<&FlagHolder>,
    mut commands: Commands,
) -> Result {
    for damage_event in damage_events.read() {
        let Ok((entity, health, owner, maybe_targeted_by, maybe_flag_assignment, maybe_army)) =
            units.get(damage_event.target_entity)
        else {
            continue;
        };
        if health.hitpoints > 0. {
            continue;
        }

        commands
            .entity(entity)
            .insert(DelayedDespawn(Timer::from_seconds(600., TimerMode::Once)))
            .despawn_related::<BehaveSources>()
            .remove::<Health>()
            .try_remove::<Interactable>();

        unit_animation.write(ToClients {
            mode: SendMode::Broadcast,
            message: AnimationChangeEvent {
                entity,
                change: AnimationChange::Death,
            },
        });

        if let Some(targeted_by) = maybe_targeted_by {
            commands
                .entity(entity)
                .remove_related::<Target>(targeted_by);
        };

        let Some(flag_assignment) = maybe_flag_assignment else {
            commands.entity(entity).remove::<BanditBehaviour>();
            continue;
        };

        commands
            .entity(entity)
            .remove::<FlagAssignment>()
            .remove::<UnitBehaviour>();

        let flag = flag_assignment.entity();
        let group = group.get(flag)?;
        let num_alive = group.len();

        // last unit from flag died
        if num_alive == 1 {
            let flag_transform = transform.get(flag)?;

            commands
                .entity(flag)
                .insert((
                    DelayedDespawn(Timer::from_seconds(620., TimerMode::Once)),
                    FlagDestroyed,
                ))
                .remove::<AttachedTo>()
                .remove::<Interactable>();

            let Ok(player) = owner.entity() else {
                continue;
            };

            if let Ok(holder) = holder.get(player)
                && flag.eq(&**holder)
            {
                commands.entity(player).remove::<FlagHolder>();
            }

            if let Some(army) = maybe_army {
                for formation_flag in army.flags.iter().flatten() {
                    commands.entity(*formation_flag).remove::<AttachedTo>();
                    commands.entity(*formation_flag).insert((
                        *flag_transform,
                        Velocity(Vec2::new((fastrand::f32() - 0.5) * 150., 100.)),
                        Visibility::Visible,
                        Interactable {
                            kind: InteractionType::Flag,
                            restricted_to: Some(player),
                        },
                    ));
                }
            }
        }
    }
    Ok(())
}
