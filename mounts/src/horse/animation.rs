use bevy::prelude::*;

use shared::{AnimationChange, AnimationChangeEvent, enum_map::*};

use animations::{
    AnimationSpriteSheet, AnimationTrigger, SpriteSheetAnimation, anim, sound::AnimationSound,
};

use crate::Mount;

pub struct HorseAnimationPlugin;

impl Plugin for HorseAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HorseSpriteSheet>()
            .add_message::<AnimationTrigger<HorseAnimation>>()
            .add_observer(init_horse_sprite)
            .add_systems(Update, (set_horse_sprite_animation, next_horse_animation));
    }
}

const ATLAS_COLUMNS: usize = 8;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
enum HorseAnimation {
    #[default]
    Idle,
    Walk,
}

#[derive(Resource)]
struct HorseSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<HorseAnimation, Image>,
}

impl FromWorld for HorseSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/animals/horse.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            ATLAS_COLUMNS as u32,
            2,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            HorseAnimation::Idle => anim!(0, 7),
            HorseAnimation::Walk => anim!(1, 5),
        });

        let animations_sound = EnumMap::new(|c| match c {
            HorseAnimation::Idle => None,
            HorseAnimation::Walk => None,
        });

        HorseSpriteSheet {
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

fn init_horse_sprite(
    trigger: On<Add, Mount>,
    mut portal: Query<&mut Sprite>,
    horse_sprite_sheet: Res<HorseSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let mut sprite = portal.get_mut(trigger.entity)?;

    let sprite_sheet = &horse_sprite_sheet.sprite_sheet;
    let animation = sprite_sheet.animations.get(HorseAnimation::default());

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((animation.clone(), HorseAnimation::default()));
    Ok(())
}

fn next_horse_animation(
    mut network_events: MessageReader<AnimationChangeEvent>,
    mut animation_trigger: MessageWriter<AnimationTrigger<HorseAnimation>>,
) {
    for event in network_events.read() {
        let new_animation = match &event.change {
            AnimationChange::Attack
            | AnimationChange::Hit(_)
            | AnimationChange::Death
            | AnimationChange::Mount
            | AnimationChange::Idle
            | AnimationChange::KnockOut
            | AnimationChange::Unmount => HorseAnimation::Idle,
        };

        animation_trigger.write(AnimationTrigger {
            entity: event.entity,
            state: new_animation,
        });
    }
}

fn set_horse_sprite_animation(
    mut command: Commands,
    mut query: Query<(
        Entity,
        &mut SpriteSheetAnimation,
        &mut Sprite,
        &mut HorseAnimation,
    )>,
    mut animation_changed: MessageReader<AnimationTrigger<HorseAnimation>>,
    horse_sprite_sheet: Res<HorseSpriteSheet>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((entity, mut sprite_animation, mut sprite, mut current_animation)) =
            query.get_mut(new_animation.entity)
        {
            let animation = horse_sprite_sheet
                .sprite_sheet
                .animations
                .get(new_animation.state);

            let sound = horse_sprite_sheet
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
