use bevy::prelude::*;

use bandits::bandit::bandit;
use humans::{archer::archer, pikeman::pikeman, shieldwarrior::shieldwarrior};
use shared::{
    enum_map::*,
    networking::UnitType,
    server::{
        entities::{Unit, UnitAnimation},
        physics::movement::Moving,
    },
    AnimationChange, AnimationChangeEvent,
};

use super::{AnimationTrigger, PlayOnce, SpriteSheet, SpriteSheetAnimation};

pub mod bandits;
pub mod humans;

#[derive(Resource)]
pub struct UnitSpriteSheets {
    pub sprite_sheets: EnumMap<UnitType, SpriteSheet<UnitAnimation>>,
}

impl FromWorld for UnitSpriteSheets {
    fn from_world(world: &mut World) -> Self {
        let shieldwarrior = shieldwarrior(world);
        let pikeman = pikeman(world);
        let archer = archer(world);
        let bandit = bandit(world);

        let sprite_sheets = EnumMap::new(|c| match c {
            UnitType::Shieldwarrior => shieldwarrior.clone(),
            UnitType::Pikeman => pikeman.clone(),
            UnitType::Archer => archer.clone(),
            UnitType::Bandit => bandit.clone(),
        });

        UnitSpriteSheets { sprite_sheets }
    }
}

pub fn trigger_unit_animation(
    mut commands: Commands,
    mut network_events: EventReader<AnimationChangeEvent>,
    mut animation_trigger: EventWriter<AnimationTrigger<UnitAnimation>>,
) {
    for event in network_events.read() {
        let new_animation = match &event.change {
            AnimationChange::Attack => UnitAnimation::Attack,
            AnimationChange::Hit => UnitAnimation::Hit,
            AnimationChange::Death => UnitAnimation::Death,
            AnimationChange::Mount => UnitAnimation::Idle,
        };

        commands.entity(event.entity).insert(PlayOnce);

        animation_trigger.send(AnimationTrigger {
            entity: event.entity,
            state: new_animation,
        });
    }
}

pub fn set_unit_walking(
    trigger: Trigger<OnAdd, Moving>,
    mut animation_trigger: EventWriter<AnimationTrigger<UnitAnimation>>,
) {
    animation_trigger.send(AnimationTrigger {
        entity: trigger.entity(),
        state: UnitAnimation::Walk,
    });
}

pub fn set_unit_after_play_once(
    trigger: Trigger<OnRemove, PlayOnce>,
    mut animation_trigger: EventWriter<AnimationTrigger<UnitAnimation>>,
    mounted: Query<&UnitAnimation>,
) {
    if let Ok(animation) = mounted.get(trigger.entity()) {
        let new_animation = match animation {
            UnitAnimation::Attack => UnitAnimation::Idle,
            _ => *animation,
        };

        animation_trigger.send(AnimationTrigger {
            entity: trigger.entity(),
            state: new_animation,
        });
    }
}

pub fn set_unit_idle(
    trigger: Trigger<OnRemove, Moving>,
    mut animation_trigger: EventWriter<AnimationTrigger<UnitAnimation>>,
) {
    animation_trigger.send(AnimationTrigger {
        entity: trigger.entity(),
        state: UnitAnimation::Idle,
    });
}

pub fn set_unit_sprite_animation(
    mut query: Query<(
        &Unit,
        &mut SpriteSheetAnimation,
        &mut Sprite,
        &mut UnitAnimation,
    )>,
    mut animation_changed: EventReader<AnimationTrigger<UnitAnimation>>,
    sprite_sheets: Res<UnitSpriteSheets>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((unit, mut sprite_animation, mut sprite, mut current_animation)) =
            query.get_mut(new_animation.entity)
        {
            let animation = sprite_sheets
                .sprite_sheets
                .get(unit.unit_type)
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
