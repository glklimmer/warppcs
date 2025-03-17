use animals::horse::{
    next_horse_animation, set_horse_sprite_animation, HorseAnimation, HorseSpriteSheet,
};
use bevy::prelude::*;

use king::{next_king_animation, set_king_sprite_animation, KingAnimation, KingSpriteSheet};
use objects::{
    chest::ChestSpriteSheet,
    flag::{FlagAnimation, FlagSpriteSheet},
};

use crate::networking::{NetworkEvent, NetworkMapping};

use shared::{
    enum_map::*,
    networking::{Facing, Hitby, Rotation, ServerMessages},
};
use units::{next_unit_animation, set_unit_sprite_animation, UnitAnimation, UnitSpriteSheets};

pub mod animals;
pub mod king;
pub mod objects;
pub mod units;

#[derive(Clone)]
pub struct SpriteSheet<E: EnumIter> {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub animations: EnumMap<E, SpriteSheetAnimation>,
    pub animations_sound: EnumMap<E, AnimationSound>,
}

#[derive(Clone)]
pub enum AnimationDirection {
    Forward,
    Backward,
}

#[derive(Component, Clone, Default, PartialEq, Eq)]
pub enum AnimationSoundTrigger {
    #[default]
    OnEnter,
    OnStartFrameTimer,
    OnEndFrameTimer,
}

#[derive(Component, Clone)]
#[require(AnimationSoundTrigger)]
pub struct AnimationSound {
    pub sound_files: Vec<String>,
    pub sound_trigger: AnimationSoundTrigger,
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

#[derive(Bundle)]
pub struct SpriteAnimationBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub initial_animation: SpriteSheetAnimation,
}

impl SpriteAnimationBundle {
    pub fn new<E: EnumIter>(
        translation: &[f32; 3],
        sprite_sheet: &SpriteSheet<E>,
        animation: E,
        scale: f32,
    ) -> Self {
        let animation = sprite_sheet.animations.get(animation);
        SpriteAnimationBundle {
            sprite: Sprite {
                image: sprite_sheet.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_sheet.layout.clone(),
                    index: animation.first_sprite_index,
                }),
                ..default()
            },
            transform: Transform {
                translation: (*translation).into(),
                scale: Vec3::splat(scale),
                ..default()
            },
            initial_animation: animation.clone(),
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
struct PlayOnce;

#[derive(Debug)]
pub enum Change {
    Rotation(Rotation),
    Movement(bool),
    Attack,
    Hit(Hitby),
    Death,
}

#[derive(Event)]
pub struct EntityChangeEvent {
    pub entity: Entity,
    pub change: Change,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnitSpriteSheets>();
        app.add_event::<AnimationTrigger<UnitAnimation>>();

        app.init_resource::<KingSpriteSheet>();
        app.add_event::<AnimationTrigger<KingAnimation>>();

        app.init_resource::<FlagSpriteSheet>();
        app.add_event::<AnimationTrigger<FlagAnimation>>();

        app.init_resource::<ChestSpriteSheet>();

        app.init_resource::<HorseSpriteSheet>();
        app.add_event::<AnimationTrigger<HorseAnimation>>();

        app.add_event::<EntityChangeEvent>();

        app.add_systems(
            FixedUpdate,
            (trigger_meele_attack, trigger_hit, trigger_death),
        );

        app.add_systems(
            Update,
            (
                (set_unit_sprite_animation, next_unit_animation),
                (set_king_sprite_animation, next_king_animation),
                (set_horse_sprite_animation, next_horse_animation),
                advance_animation,
                set_unit_facing,
                set_free_orientation,
                mirror_sprite,
            ),
        );
    }
}

fn trigger_meele_attack(
    mut network_events: EventReader<NetworkEvent>,
    mut change: EventWriter<EntityChangeEvent>,
    network_mapping: Res<NetworkMapping>,
) {
    for event in network_events.read() {
        if let ServerMessages::MeleeAttack {
            entity: server_entity,
        } = event.message
        {
            if let Some(client_entity) = network_mapping.0.get(&server_entity) {
                change.send(EntityChangeEvent {
                    entity: *client_entity,
                    change: Change::Attack,
                });
            }
        }
    }
}

fn trigger_hit(
    mut network_events: EventReader<NetworkEvent>,
    mut change: EventWriter<EntityChangeEvent>,
    network_mapping: Res<NetworkMapping>,
) {
    for event in network_events.read() {
        if let ServerMessages::EntityHit {
            entity: server_entity,
            by,
        } = event.message
        {
            if let Some(client_entity) = network_mapping.0.get(&server_entity) {
                change.send(EntityChangeEvent {
                    entity: *client_entity,
                    change: Change::Hit(by),
                });
            }
        }
    }
}

fn trigger_death(
    mut network_events: EventReader<NetworkEvent>,
    mut change: EventWriter<EntityChangeEvent>,
    network_mapping: Res<NetworkMapping>,
) {
    for event in network_events.read() {
        if let ServerMessages::EntityDeath {
            entity: server_entity,
        } = event.message
        {
            if let Some(client_entity) = network_mapping.0.get(&server_entity) {
                change.send(EntityChangeEvent {
                    entity: *client_entity,
                    change: Change::Death,
                });
            }
        }
    }
}

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
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };

        if animation.frame_timer.just_finished() {
            atlas.index = if atlas.index == animation.last_sprite_index {
                if maybe_play_once.is_some() {
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

fn set_unit_facing(mut commands: Commands, mut movements: EventReader<EntityChangeEvent>) {
    for event in movements.read() {
        if let Change::Rotation(Rotation::LeftRight {
            facing: Some(new_facing),
        }) = &event.change
        {
            if let Some(mut entity) = commands.get_entity(event.entity) {
                entity.try_insert(UnitFacing(new_facing.clone()));
            }
        }
    }
}

fn set_free_orientation(
    mut query: Query<&mut Transform>,
    mut movements: EventReader<EntityChangeEvent>,
) {
    for event in movements.read() {
        if let Change::Rotation(Rotation::Free { angle }) = &event.change {
            if let Ok(mut transform) = query.get_mut(event.entity) {
                transform.rotation = Quat::from_axis_angle(Vec3::Z, *angle);
            }
        }
    }
}

fn mirror_sprite(mut query: Query<(&UnitFacing, &mut Transform)>) {
    for (unit_facing, mut transform) in &mut query {
        let new_scale_x = match unit_facing.0 {
            Facing::Left => -transform.scale.x.abs(),
            Facing::Right => transform.scale.x.abs(),
        };
        if transform.scale.x != new_scale_x {
            transform.scale.x = new_scale_x;
        }
    }
}
