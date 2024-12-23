use bevy::prelude::*;

use shared::enum_map::*;

use super::{
    AnimationTrigger, Change, EntityChangeEvent, FullAnimation, SpriteSheet, SpriteSheetAnimation,
};

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable)]
pub enum KingAnimation {
    Idle,
    Drink,
    Walk,
    Attack,
}

#[derive(Resource)]
pub struct KingSpriteSheet {
    pub sprite_sheet: SpriteSheet<KingAnimation>,
}

impl FromWorld for KingSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/humans/Outline/MiniKingMan.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let idle = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            4,
            None,
            None,
        ));

        let drink = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            5,
            None,
            Some(UVec2::new(0, 32)),
        ));

        let walk = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            6,
            None,
            Some(UVec2::new(0, 32 * 2)),
        ));

        let attack = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            0,
            11,
            None,
            Some(UVec2::new(0, 32 * 3)),
        ));

        let animations = EnumMap::new(|c| match c {
            KingAnimation::Idle => SpriteSheetAnimation {
                layout: idle.clone(),
                first_sprite_index: 0,
                last_sprite_index: 3,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
            KingAnimation::Drink => SpriteSheetAnimation {
                layout: drink.clone(),
                first_sprite_index: 0,
                last_sprite_index: 4,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
            KingAnimation::Walk => SpriteSheetAnimation {
                layout: walk.clone(),
                first_sprite_index: 0,
                last_sprite_index: 5,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
            KingAnimation::Attack => SpriteSheetAnimation {
                layout: attack.clone(),
                first_sprite_index: 0,
                last_sprite_index: 10,
                frame_timer: Timer::from_seconds(1. / 10., TimerMode::Repeating),
            },
        });

        KingSpriteSheet {
            sprite_sheet: SpriteSheet {
                texture,
                animations,
            },
        }
    }
}

pub fn set_next_king_animation(
    mut commands: Commands,
    mut query: Query<(&mut KingAnimation, Option<&FullAnimation>)>,
    mut network_events: EventReader<EntityChangeEvent>,
    mut animation_trigger: EventWriter<AnimationTrigger<KingAnimation>>,
) {
    for event in network_events.read() {
        if let Ok((mut current_animation, maybe_full)) = query.get_mut(event.entity) {
            let maybe_new_animation = match &event.change {
                Change::Movement(moving) => match moving {
                    true => Some(KingAnimation::Walk),
                    false => Some(KingAnimation::Idle),
                },
                Change::Attack => Some(KingAnimation::Attack),
                Change::Rotation(_) => None,
            };

            if let Some(new_animation) = maybe_new_animation {
                if is_interupt_animation(&new_animation)
                    || (maybe_full.is_none() && new_animation != *current_animation)
                {
                    *current_animation = new_animation;

                    if is_full_animation(&new_animation) {
                        commands.entity(event.entity).insert(FullAnimation);
                    }
                    animation_trigger.send(AnimationTrigger {
                        entity: event.entity,
                        state: new_animation,
                    });

                    if is_full_animation(&new_animation) {
                        break;
                    }
                }
            }
        }
    }
}

fn is_interupt_animation(animation: &KingAnimation) -> bool {
    match animation {
        KingAnimation::Idle => false,
        KingAnimation::Drink => false,
        KingAnimation::Walk => false,
        KingAnimation::Attack => true,
    }
}

fn is_full_animation(animation: &KingAnimation) -> bool {
    match animation {
        KingAnimation::Idle => false,
        KingAnimation::Drink => false,
        KingAnimation::Walk => false,
        KingAnimation::Attack => true,
    }
}

pub fn set_unit_animation_layout(
    mut query: Query<(&mut SpriteSheetAnimation, &mut TextureAtlas)>,
    mut animation_changed: EventReader<AnimationTrigger<KingAnimation>>,
    king_sprite_sheet: Res<KingSpriteSheet>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((mut sprite_animation, mut atlas)) = query.get_mut(new_animation.entity) {
            let animation = king_sprite_sheet
                .sprite_sheet
                .animations
                .get(new_animation.state);

            atlas.layout = animation.layout.clone();
            atlas.index = animation.first_sprite_index;

            sprite_animation.frame_timer = animation.frame_timer.clone();
            sprite_animation.frame_timer.reset();
        }
    }
}
