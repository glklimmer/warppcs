use bevy::prelude::*;

use shared::{
    enum_map::*,
    networking::{MountType, Mounted},
    server::physics::movement::Moving,
    AnimationChange, AnimationChangeEvent,
};

use super::{AnimationTrigger, PlayOnce, SpriteSheet, SpriteSheetAnimation};

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
pub enum KingAnimation {
    #[default]
    Idle,
    Drink,
    Walk,
    Attack,
    Hit,
    Death,
    Mount,
    HorseIdle,
    HorseWalk,
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
            10,
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
            KingAnimation::Mount => SpriteSheetAnimation {
                first_sprite_index: 70,
                last_sprite_index: 76,
                ..default()
            },
            KingAnimation::HorseIdle => SpriteSheetAnimation {
                first_sprite_index: 80,
                last_sprite_index: 87,
                ..default()
            },
            KingAnimation::HorseWalk => SpriteSheetAnimation {
                first_sprite_index: 90,
                last_sprite_index: 95,
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

pub fn trigger_king_animation(
    mut animation_changes: EventReader<AnimationChangeEvent>,
    mut animation_trigger: EventWriter<AnimationTrigger<KingAnimation>>,
    mut commands: Commands,
    mounted: Query<Option<&Mounted>>,
) {
    for event in animation_changes.read() {
        if let Ok(maybe_mounted) = mounted.get(event.entity) {
            let new_animation = match maybe_mounted {
                Some(_) => todo!(),
                None => match &event.change {
                    AnimationChange::Attack => KingAnimation::Attack,
                    AnimationChange::Hit => KingAnimation::Hit,
                    AnimationChange::Death => KingAnimation::Death,
                },
            };

            commands.entity(event.entity).insert(PlayOnce);

            animation_trigger.send(AnimationTrigger {
                entity: event.entity,
                state: new_animation,
            });
        }
    }
}

pub fn set_king_walking(
    trigger: Trigger<OnAdd, Moving>,
    mut animation_trigger: EventWriter<AnimationTrigger<KingAnimation>>,
    mounted: Query<Option<&Mounted>>,
) {
    if let Ok(maybe_mounted) = mounted.get(trigger.entity()) {
        let new_animation = match maybe_mounted {
            Some(_) => KingAnimation::HorseWalk,
            None => KingAnimation::Walk,
        };

        animation_trigger.send(AnimationTrigger {
            entity: trigger.entity(),
            state: new_animation,
        });
    }
}

pub fn set_king_idle(
    trigger: Trigger<OnRemove, Moving>,
    mut animation_trigger: EventWriter<AnimationTrigger<KingAnimation>>,
    mounted: Query<Option<&Mounted>>,
) {
    if let Ok(maybe_mounted) = mounted.get(trigger.entity()) {
        let new_animation = match maybe_mounted {
            Some(_) => KingAnimation::HorseIdle,
            None => KingAnimation::Idle,
        };

        animation_trigger.send(AnimationTrigger {
            entity: trigger.entity(),
            state: new_animation,
        });
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
