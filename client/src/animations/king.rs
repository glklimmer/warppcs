use bevy::prelude::*;

use shared::enum_map::*;

use super::{
    AnimationTrigger, Change, EntityChangeEvent, FullAnimation, SpriteSheet, SpriteSheetAnimation,
};

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum KingAnimation {
    #[default]
    Idle,
    Drink,
    Walk,
    Attack,
    Hit,
    Death,
}

#[derive(Resource)]
pub struct KingSpriteSheet {
    pub sprite_sheet: SpriteSheet<KingAnimation>,
}

impl FromWorld for KingSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/humans/MiniKingMan.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            10,
            7,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            KingAnimation::Idle => SpriteSheetAnimation {
                first_sprite_index: 0,
                last_sprite_index: 3,
                ..default()
            },
            KingAnimation::Drink => SpriteSheetAnimation {
                first_sprite_index: 10,
                last_sprite_index: 14,
                ..default()
            },
            KingAnimation::Walk => SpriteSheetAnimation {
                first_sprite_index: 20,
                last_sprite_index: 25,
                ..default()
            },
            KingAnimation::Attack => SpriteSheetAnimation {
                first_sprite_index: 40,
                last_sprite_index: 49,
                ..default()
            },
            KingAnimation::Hit => SpriteSheetAnimation {
                first_sprite_index: 50,
                last_sprite_index: 52,
                ..default()
            },
            KingAnimation::Death => SpriteSheetAnimation {
                first_sprite_index: 60,
                last_sprite_index: 65,
                ..default()
            },
        });

        KingSpriteSheet {
            sprite_sheet: SpriteSheet {
                texture,
                layout,
                animations,
            },
        }
    }
}

pub fn next_king_animation(
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
                Change::Hit => Some(KingAnimation::Hit),
                Change::Death => Some(KingAnimation::Death),
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
        KingAnimation::Hit => false,
        KingAnimation::Death => true,
    }
}

fn is_full_animation(animation: &KingAnimation) -> bool {
    match animation {
        KingAnimation::Idle => false,
        KingAnimation::Drink => false,
        KingAnimation::Walk => false,
        KingAnimation::Attack => true,
        KingAnimation::Hit => false,
        KingAnimation::Death => true,
    }
}

pub fn set_king_sprite_animation(
    mut query: Query<(&mut SpriteSheetAnimation, &mut Sprite)>,
    mut animation_changed: EventReader<AnimationTrigger<KingAnimation>>,
    king_sprite_sheet: Res<KingSpriteSheet>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((mut sprite_animation, mut sprite)) = query.get_mut(new_animation.entity) {
            let animation = king_sprite_sheet
                .sprite_sheet
                .animations
                .get(new_animation.state);

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = animation.first_sprite_index;
            }
            *sprite_animation = animation.clone();
        }
    }
}
