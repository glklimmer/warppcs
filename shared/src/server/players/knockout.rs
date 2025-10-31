use bevy::prelude::*;
use bevy_replicon::prelude::{SendMode, ToClients};

use crate::{
    AnimationChange, AnimationChangeEvent, Owner, Player, PlayerState,
    map::{
        Layers,
        buildings::{Building, BuildingType},
    },
    server::{
        ai::{Target, TargetedBy},
        buildings::recruiting::{Flag, FlagHolder},
        entities::{
            Unit,
            health::{Health, TakeDamage},
        },
        game_scenes::GameSceneId,
        physics::{attachment::AttachedTo, movement::Velocity},
    },
};

#[derive(Component)]
pub struct RespawnTimer {
    pub timer: Timer,
}

impl Default for RespawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1., TimerMode::Once),
        }
    }
}

pub struct KnockoutPlugin;

impl Plugin for KnockoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, kill_player);
        app.add_systems(
            FixedUpdate,
            respawm_player.run_if(in_state(PlayerState::Respawn)),
        );
    }
}

fn kill_player(
    mut damage_events: EventReader<TakeDamage>,
    mut player: Query<
        (
            Entity,
            &mut Transform,
            &Owner,
            &Health,
            Option<&TargetedBy>,
            Option<&FlagHolder>,
        ),
        (With<Player>, Without<Building>),
    >,
    main_building: Query<
        (&Transform, &Owner, &Building, &GameSceneId),
        (With<Health>, Without<Unit>),
    >,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut king_animation: EventWriter<ToClients<AnimationChangeEvent>>,
    transform: Query<&Transform, (With<Flag>, Without<Player>)>,
    mut commands: Commands,
) -> Result {
    for damage_event in damage_events.read() {
        let Ok((
            player_entity,
            mut player_transform,
            player,
            player_health,
            maybe_targeted_by,
            maybe_flag_holder,
        )) = player.get_mut(damage_event.target_entity)
        else {
            continue;
        };

        if player_health.hitpoints > 0. {
            continue;
        }

        if let Some(targeted_by) = maybe_targeted_by {
            commands
                .entity(player_entity)
                .remove_related::<Target>(targeted_by);
        };

        king_animation.write(ToClients {
            mode: SendMode::Broadcast,
            event: AnimationChangeEvent {
                entity: player_entity,
                change: AnimationChange::KnockOut,
            },
        });

        if let Some(flag) = maybe_flag_holder {
            let flag_transform = transform.get(**flag)?;

            commands.entity(**flag).remove::<AttachedTo>();
            commands.entity(**flag).insert((
                *flag_transform,
                Velocity(Vec2::new((fastrand::f32() - 0.5) * 150., 100.)),
                Visibility::Visible,
            ));
        }

        for (building_transform, owner, building, building_scene) in main_building.iter() {
            if let BuildingType::MainBuilding { level: _ } = building.building_type
                && owner.is_same_faction(player)
            {
                commands
                    .entity(player_entity)
                    .remove::<(FlagHolder, Health)>()
                    .insert((*building_scene, RespawnTimer::default()));

                player_transform.translation = building_transform
                    .translation
                    .with_z(Layers::Player.as_f32());

                next_state.set(PlayerState::Respawn);
                break;
            }
        }
    }
    Ok(())
}

fn respawm_player(
    mut respawn_query: Query<(Entity, &mut RespawnTimer)>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut king_animation: EventWriter<ToClients<AnimationChangeEvent>>,
    mut commands: Commands,
) -> Result {
    for (player_entity, mut timer) in respawn_query.iter_mut() {
        timer.timer.tick(time.delta());

        if timer.timer.just_finished() {
            commands
                .entity(player_entity)
                .insert(Health { hitpoints: 200. });

            next_state.set(PlayerState::World);

            king_animation.write(ToClients {
                mode: SendMode::Broadcast,
                event: AnimationChangeEvent {
                    entity: player_entity,
                    change: AnimationChange::Idle,
                },
            });
        }
    }
    Ok(())
}
