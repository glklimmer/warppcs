use bevy::prelude::*;

use animals::horse::{
    next_horse_animation, set_horse_sprite_animation, HorseAnimation, HorseSpriteSheet,
};
use bevy_replicon::client::ClientSet;
use king::{
    set_king_after_play_once, set_king_idle, set_king_sprite_animation, set_king_walking,
    trigger_king_animation, KingAnimation, KingSpriteSheet,
};
use objects::{
    chest::ChestSpriteSheet,
    flag::{FlagAnimation, FlagSpriteSheet},
    portal::PortalSpriteSheet,
};
use shared::{enum_map::*, networking::Facing, server::entities::UnitAnimation};
use units::{
    set_unit_after_play_once, set_unit_idle, set_unit_sprite_animation, set_unit_walking,
    trigger_unit_animation, UnitSpriteSheets,
};

pub mod animals;
pub mod king;
pub mod objects;
pub mod units;

#[derive(Clone)]
pub struct SpriteSheet<E: EnumIter> {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub animations: EnumMap<E, SpriteSheetAnimation>,
}

#[derive(Clone)]
pub enum AnimationDirection {
    Forward,
    Backward,
}

#[derive(Component, Clone)]
pub struct SpriteSheetAnimation {
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub frame_timer: Timer,
    pub direction: AnimationDirection,
}

impl Default for SpriteSheetAnimation {
    fn default() -> Self {
        SpriteSheetAnimation {
            first_sprite_index: 0,
            last_sprite_index: 0,
            frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            direction: AnimationDirection::Forward,
        }
    }
}

#[derive(Component)]
pub struct UnitFacing(pub Facing);

/// Gets only triggered if new animation
#[derive(Debug, Event)]
pub struct AnimationTrigger<E> {
    pub entity: Entity,
    pub state: E,
}

#[derive(Component)]
pub struct FullAnimation;

#[derive(Component)]
pub struct PlayOnce;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnitSpriteSheets>();
        app.add_event::<AnimationTrigger<UnitAnimation>>();

        app.init_resource::<KingSpriteSheet>();
        app.add_event::<AnimationTrigger<KingAnimation>>();

        app.init_resource::<FlagSpriteSheet>();
        app.init_resource::<ChestSpriteSheet>();
        app.init_resource::<PortalSpriteSheet>();

        app.init_resource::<HorseSpriteSheet>();
        app.add_event::<AnimationTrigger<HorseAnimation>>();

        // app.add_event::<EntityChangeEvent>();

        // app.add_systems(
        //     FixedUpdate,
        //     (trigger_meele_attack, trigger_hit, trigger_death),
        // );

        app.add_systems(
            PreUpdate,
            (trigger_king_animation, trigger_unit_animation).after(ClientSet::Receive),
        )
        .add_observer(set_king_walking)
        .add_observer(set_king_idle)
        .add_observer(set_king_after_play_once)
        .add_observer(set_unit_walking)
        .add_observer(set_unit_idle)
        .add_observer(set_unit_after_play_once);

        app.add_systems(
            Update,
            (
                (set_unit_sprite_animation),
                (set_king_sprite_animation),
                (set_horse_sprite_animation, next_horse_animation),
                advance_animation,
                // set_unit_facing,
                // set_free_orientation,
                // mirror_sprite,
            ),
        );
    }
}

// fn trigger_meele_attack(
//     mut network_events: EventReader<NetworkEvent>,
//     mut change: EventWriter<EntityChangeEvent>,
//     network_mapping: Res<NetworkMapping>,
// ) {
//     for event in network_events.read() {
//         if let ServerMessages::MeleeAttack {
//             entity: server_entity,
//         } = event.message
//         {
//             if let Some(client_entity) = network_mapping.0.get(&server_entity) {
//                 change.send(EntityChangeEvent {
//                     entity: *client_entity,
//                     change: Change::Attack,
//                 });
//             }
//         }
//     }
// }
//
// fn trigger_hit(
//     mut network_events: EventReader<NetworkEvent>,
//     mut change: EventWriter<EntityChangeEvent>,
//     network_mapping: Res<NetworkMapping>,
// ) {
//     for event in network_events.read() {
//         if let ServerMessages::EntityHit {
//             entity: server_entity,
//         } = event.message
//         {
//             if let Some(client_entity) = network_mapping.0.get(&server_entity) {
//                 change.send(EntityChangeEvent {
//                     entity: *client_entity,
//                     change: Change::Hit,
//                 });
//             }
//         }
//     }
// }
//
// fn trigger_death(
//     mut network_events: EventReader<NetworkEvent>,
//     mut change: EventWriter<EntityChangeEvent>,
//     network_mapping: Res<NetworkMapping>,
// ) {
//     for event in network_events.read() {
//         if let ServerMessages::EntityDeath {
//             entity: server_entity,
//         } = event.message
//         {
//             if let Some(client_entity) = network_mapping.0.get(&server_entity) {
//                 change.send(EntityChangeEvent {
//                     entity: *client_entity,
//                     change: Change::Death,
//                 });
//             }
//         }
//     }
// }

#[allow(clippy::type_complexity)]
fn advance_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut SpriteSheetAnimation,
        &mut Sprite,
        Option<&FullAnimation>,
        Option<&PlayOnce>,
    )>,
) {
    for (entity, mut animation, mut sprite, maybe_full, maybe_play_once) in &mut query {
        animation.frame_timer.tick(time.delta());
        let atlas = sprite.texture_atlas.as_mut().unwrap();

        if animation.frame_timer.just_finished() {
            atlas.index = if atlas.index == animation.last_sprite_index {
                if maybe_play_once.is_some() {
                    commands.entity(entity).remove::<PlayOnce>();
                    return;
                }
                if maybe_full.is_some() {
                    commands.entity(entity).remove::<FullAnimation>();
                }
                animation.first_sprite_index
            } else {
                match animation.direction {
                    AnimationDirection::Forward => atlas.index + 1,
                    AnimationDirection::Backward => atlas.index - 1,
                }
            };
        }
    }
}

// fn set_unit_facing(mut commands: Commands, mut movements: EventReader<EntityChangeEvent>) {
//     for event in movements.read() {
//         if let Change::Rotation(Rotation::LeftRight {
//             facing: Some(new_facing),
//         }) = &event.change
//         {
//             if let Some(mut entity) = commands.get_entity(event.entity) {
//                 entity.try_insert(UnitFacing(new_facing.clone()));
//             }
//         }
//     }
// }
//
// fn set_free_orientation(
//     mut query: Query<&mut Transform>,
//     mut movements: EventReader<EntityChangeEvent>,
// ) {
//     for event in movements.read() {
//         if let Change::Rotation(Rotation::Free { angle }) = &event.change {
//             if let Ok(mut transform) = query.get_mut(event.entity) {
//                 transform.rotation = Quat::from_axis_angle(Vec3::Z, *angle);
//             }
//         }
//     }
// }
//
// fn mirror_sprite(mut query: Query<(&UnitFacing, &mut Transform)>) {
//     for (unit_facing, mut transform) in &mut query {
//         let new_scale_x = match unit_facing.0 {
//             Facing::Left => -transform.scale.x.abs(),
//             Facing::Right => transform.scale.x.abs(),
//         };
//         if transform.scale.x != new_scale_x {
//             transform.scale.x = new_scale_x;
//         }
//     }
// }
