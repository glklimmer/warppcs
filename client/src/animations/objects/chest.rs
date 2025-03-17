use bevy::prelude::*;

use shared::enum_map::*;

use crate::animations::{AnimationDirection, AnimationSound, AnimationSoundTrigger};

use super::super::{SpriteSheet, SpriteSheetAnimation};

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable)]
pub enum ChestAnimation {
    Open,
    Close,
}

#[derive(Resource)]
pub struct ChestSpriteSheet {
    pub sprite_sheet: SpriteSheet<ChestAnimation>,
}

impl FromWorld for ChestSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/objects/chest.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(32, 32),
            3,
            2,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            ChestAnimation::Open => SpriteSheetAnimation {
                first_sprite_index: 0,
                last_sprite_index: 2,
                ..default()
            },
            ChestAnimation::Close => SpriteSheetAnimation {
                first_sprite_index: 2,
                last_sprite_index: 0,
                direction: AnimationDirection::Backward,
                ..default()
            },
        });

        let animations_sound = EnumMap::new(|c| match c {
            ChestAnimation::Open => AnimationSound {
                sound_files: vec![],
                sound_trigger: AnimationSoundTrigger::OnEnter,
            },
            ChestAnimation::Close => AnimationSound {
                sound_files: vec![],
                sound_trigger: AnimationSoundTrigger::OnEnter,
            },
        });

        ChestSpriteSheet {
            sprite_sheet: SpriteSheet {
                texture,
                layout,
                animations,
                animations_sound,
            },
        }
    }
}

// pub fn next_chest_animation(
//     mut commands: Commands,
//     mut query: Query<(&mut ChestAnimation)>,
//     mut network_events: EventReader<EntityChangeEvent>,
//     mut animation_trigger: EventWriter<AnimationTrigger<ChestAnimation>>,
// ) {
//     for event in network_events.read() {
//         if let Ok((mut current_animation, maybe_full)) = query.get_mut(event.entity) {
//             let maybe_new_animation = match &event.change {
//                 Change::Movement(moving) => match moving {
//                     true => Some(KingAnimation::Walk),
//                     false => Some(KingAnimation::Idle),
//                 },
//                 Change::Attack => Some(KingAnimation::Attack),
//                 Change::Rotation(_) => None,
//                 Change::Hit => Some(KingAnimation::Hit),
//                 Change::Death => Some(KingAnimation::Death),
//             };
//
//             if let Some(new_animation) = maybe_new_animation {
//                 if is_interupt_animation(&new_animation)
//                     || (maybe_full.is_none() && new_animation != *current_animation)
//                 {
//                     *current_animation = new_animation;
//
//                     if is_full_animation(&new_animation) {
//                         commands.entity(event.entity).insert(FullAnimation);
//                     }
//                     animation_trigger.send(AnimationTrigger {
//                         entity: event.entity,
//                         state: new_animation,
//                     });
//
//                     if is_full_animation(&new_animation) {
//                         break;
//                     }
//                 }
//             }
//         }
//     }
// }
