use bevy::prelude::*;

use shared::{AnimationChange, AnimationChangeEvent, enum_map::*};

use crate::animations::{AnimationTrigger, SpriteSheet, SpriteSheetAnimation};

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum HorseAnimation {
    #[default]
    Idle,
    Walk,
}

#[derive(Resource)]
pub struct HorseSpriteSheet {
    pub sprite_sheet: SpriteSheet<HorseAnimation>,
}

impl FromWorld for HorseSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/animals/horse.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            8,
            2,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            HorseAnimation::Idle => SpriteSheetAnimation {
                first_sprite_index: 0,
                last_sprite_index: 7,
                ..default()
            },
            HorseAnimation::Walk => SpriteSheetAnimation {
                first_sprite_index: 8,
                last_sprite_index: 13,
                ..default()
            },
        });

        HorseSpriteSheet {
            sprite_sheet: SpriteSheet {
                texture,
                layout,
                animations,
            },
        }
    }
}

pub fn next_horse_animation(
    mut network_events: EventReader<AnimationChangeEvent>,
    mut animation_trigger: EventWriter<AnimationTrigger<HorseAnimation>>,
) {
    for event in network_events.read() {
        let new_animation = match &event.change {
            AnimationChange::Attack
            | AnimationChange::Hit
            | AnimationChange::Death
            | AnimationChange::Mount => HorseAnimation::Idle,
        };

        animation_trigger.send(AnimationTrigger {
            entity: event.entity,
            state: new_animation,
        });
    }
}

pub fn set_horse_sprite_animation(
    mut query: Query<(&mut SpriteSheetAnimation, &mut Sprite, &mut HorseAnimation)>,
    mut animation_changed: EventReader<AnimationTrigger<HorseAnimation>>,
    king_sprite_sheet: Res<HorseSpriteSheet>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((mut sprite_animation, mut sprite, mut current_animation)) =
            query.get_mut(new_animation.entity)
        {
            let animation = king_sprite_sheet
                .sprite_sheet
                .animations
                .get(new_animation.state);

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = animation.first_sprite_index;
            }
            *sprite_animation = animation.clone();
            *current_animation = new_animation.state;
        }
    }
}
