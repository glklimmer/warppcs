use bevy::prelude::*;
use bevy::time::Timer;

use bevy::sprite::Anchor;
use bevy_replicon::prelude::{Replicated, SendMode, ToClients};
use serde::{Deserialize, Serialize};

use crate::{
    AnimationChange, AnimationChangeEvent, BoxCollider,
    map::Layers,
    networking::{MountType, Mounted},
    server::{
        game_scenes::GameSceneId,
        physics::movement::{Speed, Velocity},
    },
    unit_collider,
};

use super::interaction::{Interactable, InteractionTriggeredEvent, InteractionType};

pub struct MountPlugin;

impl Plugin for MountPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                spawn_mount_on_unmount,
                (mount, unmount).run_if(on_event::<InteractionTriggeredEvent>),
            ),
        );
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = unit_collider(),
    Velocity,
    Sprite{anchor: Anchor::BottomCenter, ..default()},
    Interactable{
        kind: InteractionType::Mount,
        restricted_to: None,
    },
)]
pub struct Mount {
    pub mount_type: MountType,
}

#[derive(Component)]
struct DelayedMountSpawn {
    mount_type: MountType,
    position: Transform,
    timer: Timer,
}

impl From<MountType> for Speed {
    fn from(value: MountType) -> Self {
        match value {
            MountType::Horse => Speed(150.),
        }
    }
}

fn mount(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut commands: Commands,
    mut animation: EventWriter<ToClients<AnimationChangeEvent>>,
    mount_query: Query<&Mount>,
) -> Result {
    for event in interactions.read() {
        let InteractionType::Mount = &event.interaction else {
            continue;
        };

        let player = event.player;
        let mount = mount_query.get(event.interactable)?;

        let new_speed: Speed = mount.mount_type.into();

        commands.entity(event.interactable).despawn();
        commands.entity(player).insert((
            Mounted {
                mount_type: mount.mount_type,
            },
            new_speed,
            Interactable {
                kind: InteractionType::Unmount,
                restricted_to: Some(event.player),
            },
        ));

        animation.write(ToClients {
            mode: SendMode::Broadcast,
            event: AnimationChangeEvent {
                entity: player,
                change: AnimationChange::Mount,
            },
        });
    }
    Ok(())
}

fn unmount(
    mut interactions: EventReader<InteractionTriggeredEvent>,
    mut player_query: Query<(&Mounted, &Transform, &GameSceneId)>,
    mut animation: EventWriter<ToClients<AnimationChangeEvent>>,
    mut commands: Commands,
) -> Result {
    for event in interactions.read() {
        let InteractionType::Unmount = &event.interaction else {
            continue;
        };

        let player = event.player;
        let (mounted, transform, game_scene_id) = player_query.get_mut(player)?;

        let new_speed = Speed::default();

        commands
            .entity(event.player)
            .remove::<Mounted>()
            .remove::<Interactable>()
            .insert(new_speed);

        commands.spawn((
            DelayedMountSpawn {
                mount_type: mounted.mount_type,
                position: transform
                    .with_translation(transform.translation.with_z(Layers::Mount.as_f32())),
                timer: Timer::from_seconds(0.1 * 7., TimerMode::Once), // TODO: replace with animation
                                                                       // hook
            },
            *game_scene_id,
        ));

        animation.write(ToClients {
            mode: SendMode::Broadcast,
            event: AnimationChangeEvent {
                entity: player,
                change: AnimationChange::Unmount,
            },
        });
    }
    Ok(())
}

fn spawn_mount_on_unmount(
    mut delayed_spawns: Query<(Entity, &mut DelayedMountSpawn, &GameSceneId)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut delayed_spawn, game_scene_id) in delayed_spawns.iter_mut() {
        delayed_spawn.timer.tick(time.delta());

        if delayed_spawn.timer.finished() {
            commands.spawn((
                Mount {
                    mount_type: delayed_spawn.mount_type,
                },
                delayed_spawn.position,
                *game_scene_id,
            ));

            commands.entity(entity).despawn();
        }
    }
}
