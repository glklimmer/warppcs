use bevy::prelude::*;

use archer::archer;
use shared::{enum_map::*, networking::UnitType, server::entities::Unit};

use super::{
    AnimationTrigger, Change, EntityChangeEvent, FullAnimation, SpriteSheet, SpriteSheetAnimation,
};

pub mod archer;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable)]
pub enum UnitAnimation {
    Idle,
    Walk,
    Attack,
}

#[derive(Resource)]
pub struct UnitSpriteSheets {
    pub sprite_sheets: EnumMap<UnitType, SpriteSheet<UnitAnimation>>,
}

impl FromWorld for UnitSpriteSheets {
    fn from_world(world: &mut World) -> Self {
        let archer = archer(world);

        let sprite_sheets = EnumMap::new(|c| match c {
            UnitType::Shieldwarrior => archer.clone(),
            UnitType::Pikeman => archer.clone(),
            UnitType::Archer => archer.clone(),
        });

        UnitSpriteSheets { sprite_sheets }
    }
}

pub fn set_next_unit_animation(
    mut commands: Commands,
    mut query: Query<(&mut UnitAnimation, Option<&FullAnimation>)>,
    mut network_events: EventReader<EntityChangeEvent>,
    mut animation_trigger: EventWriter<AnimationTrigger<UnitAnimation>>,
) {
    for event in network_events.read() {
        if let Ok((mut current_animation, maybe_full)) = query.get_mut(event.entity) {
            let maybe_new_animation = match &event.change {
                Change::Movement(moving) => match moving {
                    true => Some(UnitAnimation::Walk),
                    false => Some(UnitAnimation::Idle),
                },
                Change::Attack => Some(UnitAnimation::Attack),
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

fn is_interupt_animation(animation: &UnitAnimation) -> bool {
    match animation {
        UnitAnimation::Idle => false,
        UnitAnimation::Walk => false,
        UnitAnimation::Attack => true,
    }
}

fn is_full_animation(animation: &UnitAnimation) -> bool {
    match animation {
        UnitAnimation::Idle => false,
        UnitAnimation::Walk => false,
        UnitAnimation::Attack => true,
    }
}

pub fn set_unit_animation_layout(
    mut query: Query<(&Unit, &mut SpriteSheetAnimation, &mut TextureAtlas)>,
    mut animation_changed: EventReader<AnimationTrigger<UnitAnimation>>,
    sprite_sheets: Res<UnitSpriteSheets>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((unit, mut sprite_animation, mut atlas)) = query.get_mut(new_animation.entity) {
            let animation = sprite_sheets
                .sprite_sheets
                .get(unit.unit_type)
                .animations
                .get(new_animation.state);

            atlas.layout = animation.layout.clone();
            atlas.index = animation.first_sprite_index;

            sprite_animation.frame_timer = animation.frame_timer.clone();
            sprite_animation.frame_timer.reset();
        }
    }
}
