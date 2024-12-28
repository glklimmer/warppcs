use bevy::prelude::*;

use archer::archer;
use pikeman::pikeman;
use shared::{enum_map::*, networking::UnitType};
use shieldwarrior::shieldwarrior;

use super::{
    AnimationTrigger, Change, EntityChangeEvent, FullAnimation, SpriteSheet, SpriteSheetAnimation,
};

pub mod archer;
pub mod pikeman;
pub mod shieldwarrior;

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
        let shieldwarrior = shieldwarrior(world);
        let pikeman = pikeman(world);
        let archer = archer(world);

        let sprite_sheets = EnumMap::new(|c| match c {
            UnitType::Shieldwarrior => shieldwarrior.clone(),
            UnitType::Pikeman => pikeman.clone(),
            UnitType::Archer => archer.clone(),
        });

        UnitSpriteSheets { sprite_sheets }
    }
}

pub fn next_unit_animation(
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

pub fn set_unit_sprite_animation(
    mut query: Query<(&UnitType, &mut SpriteSheetAnimation, &mut TextureAtlas)>,
    mut animation_changed: EventReader<AnimationTrigger<UnitAnimation>>,
    sprite_sheets: Res<UnitSpriteSheets>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((unit_type, mut sprite_animation, mut atlas)) =
            query.get_mut(new_animation.entity)
        {
            let animation = sprite_sheets
                .sprite_sheets
                .get(*unit_type)
                .animations
                .get(new_animation.state);

            atlas.index = animation.first_sprite_index;
            *sprite_animation = animation.clone();
        }
    }
}