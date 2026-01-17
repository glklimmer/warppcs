use bevy::prelude::*;

use animations::{
    AnimationSpriteSheet, AnimationTrigger, SpriteSheetAnimation, anim, sound::AnimationSound,
};
use physics::movement::Moving;
use shared::{AnimationChange, AnimationChangeEvent, enum_map::*};

use crate::Transport;

const ATLAS_COLUMNS: usize = 6;

pub(crate) struct TransporterAnimationPlugin;

impl Plugin for TransporterAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransportSpriteSheet>()
            .add_message::<AnimationTrigger<TransportAnimation>>()
            .add_observer(init_transport_sprite)
            .add_observer(set_transport_walking)
            .add_observer(set_transport_idle)
            .add_systems(
                Update,
                (set_transport_sprite_animation, next_transport_animation),
            );
    }
}

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
enum TransportAnimation {
    #[default]
    Idle,
    Walk,
}

#[derive(Resource)]
struct TransportSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<TransportAnimation, Image>,
}

impl FromWorld for TransportSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let path = match fastrand::bool() {
            true => "sprites/humans/MiniVillagerMan.png",
            false => "sprites/humans/MiniVillagerWoman.png",
        };
        let texture = asset_server.load(path);

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            ATLAS_COLUMNS as u32,
            7,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            TransportAnimation::Idle => anim!(0, 3),
            TransportAnimation::Walk => anim!(1, 5),
        });

        let animations_sound = EnumMap::new(|c| match c {
            TransportAnimation::Idle => None,
            TransportAnimation::Walk => None,
        });

        TransportSpriteSheet {
            sprite_sheet: AnimationSpriteSheet::new(
                world,
                texture,
                layout,
                animations,
                animations_sound,
            ),
        }
    }
}

fn init_transport_sprite(
    trigger: On<Add, Transport>,
    mut transport: Query<&mut Sprite>,
    transport_sprite_sheet: Res<TransportSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let mut sprite = transport.get_mut(trigger.entity)?;

    let sprite_sheet = &transport_sprite_sheet.sprite_sheet;
    let animation = sprite_sheet.animations.get(TransportAnimation::default());

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((animation.clone(), TransportAnimation::default()));
    Ok(())
}

fn set_transport_walking(
    trigger: On<Add, Moving>,
    is_transport: Query<Entity, With<Transport>>,
    mut animation_trigger: MessageWriter<AnimationTrigger<TransportAnimation>>,
) {
    if is_transport.get(trigger.entity).is_ok() {
        animation_trigger.write(AnimationTrigger {
            entity: trigger.entity,
            state: TransportAnimation::Walk,
        });
    }
}

fn set_transport_idle(
    trigger: On<Remove, Moving>,
    is_transport: Query<Entity, With<Transport>>,
    mut animation_trigger: MessageWriter<AnimationTrigger<TransportAnimation>>,
) {
    if is_transport.get(trigger.entity).is_ok() {
        animation_trigger.write(AnimationTrigger {
            entity: trigger.entity,
            state: TransportAnimation::Idle,
        });
    }
}

fn next_transport_animation(
    mut network_events: MessageReader<AnimationChangeEvent>,
    mut animation_trigger: MessageWriter<AnimationTrigger<TransportAnimation>>,
) {
    for event in network_events.read() {
        let new_animation = match &event.change {
            AnimationChange::Attack
            | AnimationChange::Hit(_)
            | AnimationChange::Death
            | AnimationChange::Mount
            | AnimationChange::Idle
            | AnimationChange::KnockOut
            | AnimationChange::Unmount => TransportAnimation::Idle,
        };

        animation_trigger.write(AnimationTrigger {
            entity: event.entity,
            state: new_animation,
        });
    }
}

fn set_transport_sprite_animation(
    mut command: Commands,
    mut query: Query<(
        Entity,
        &mut SpriteSheetAnimation,
        &mut Sprite,
        &mut TransportAnimation,
    )>,
    mut animation_changed: MessageReader<AnimationTrigger<TransportAnimation>>,
    transport_sprite_sheet: Res<TransportSpriteSheet>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((entity, mut sprite_animation, mut sprite, mut current_animation)) =
            query.get_mut(new_animation.entity)
        {
            let animation = transport_sprite_sheet
                .sprite_sheet
                .animations
                .get(new_animation.state);

            let sound = transport_sprite_sheet
                .sprite_sheet
                .animations_sound
                .get(new_animation.state);

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = animation.first_sprite_index;
            }

            match sound {
                Some(sound) => {
                    command.entity(entity).insert(sound.clone());
                }
                None => {
                    command.entity(entity).remove::<AnimationSound>();
                }
            }

            *sprite_animation = animation.clone();
            *current_animation = new_animation.state;
        }
    }
}
