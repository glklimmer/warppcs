use bevy::prelude::*;

use bandits::bandit::bandit;
use humans::{
    archer::archer, commander::commander, pikeman::pikeman, shieldwarrior::shieldwarrior,
};
use shared::{
    AnimationChange, AnimationChangeEvent,
    enum_map::*,
    networking::UnitType,
    server::{
        entities::{Unit, UnitAnimation},
        physics::movement::Moving,
    },
};

use super::{
    AnimationSound, AnimationSpriteSheet, AnimationTrigger, PlayOnce, SpriteSheetAnimation,
    sprite_variant_loader::SpriteVariants,
};

pub mod bandits;
pub mod humans;

#[derive(Resource)]
pub struct UnitSpriteSheets {
    pub sprite_sheets: EnumMap<UnitType, AnimationSpriteSheet<UnitAnimation, SpriteVariants>>,
}

impl FromWorld for UnitSpriteSheets {
    fn from_world(world: &mut World) -> Self {
        let shieldwarrior = shieldwarrior(world);
        let pikeman = pikeman(world);
        let archer = archer(world);
        let bandit = bandit(world);
        let commander = commander(world);

        let sprite_sheets = EnumMap::new(|c| match c {
            UnitType::Shieldwarrior => shieldwarrior.clone(),
            UnitType::Pikeman => pikeman.clone(),
            UnitType::Archer => archer.clone(),
            UnitType::Bandit => bandit.clone(),
            UnitType::Commander => commander.clone(),
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
            AnimationChange::Hit(_) => UnitAnimation::Hit,
            AnimationChange::Death => UnitAnimation::Death,
            AnimationChange::Mount => UnitAnimation::Idle,
            AnimationChange::Unmount => UnitAnimation::Idle,
        };

        commands.entity(event.entity).insert(PlayOnce);

        animation_trigger.write(AnimationTrigger {
            entity: event.entity,
            state: new_animation,
        });
    }
}

pub fn set_unit_walking(
    trigger: Trigger<OnAdd, Moving>,
    mut animation_trigger: EventWriter<AnimationTrigger<UnitAnimation>>,
) {
    animation_trigger.write(AnimationTrigger {
        entity: trigger.target(),
        state: UnitAnimation::Walk,
    });
}

pub fn set_unit_after_play_once(
    trigger: Trigger<OnRemove, PlayOnce>,
    mut commands: Commands,
    mut animation_trigger: EventWriter<AnimationTrigger<UnitAnimation>>,
    unit_animation: Query<&UnitAnimation>,
) {
    if let Ok(animation) = unit_animation.get(trigger.target()) {
        let mut entity = commands.entity(trigger.target());
        if let UnitAnimation::Death = animation {
            entity.remove::<SpriteSheetAnimation>();
            return;
        }

        let new_animation = match animation {
            UnitAnimation::Attack | UnitAnimation::Hit => UnitAnimation::Idle,
            _ => *animation,
        };

        animation_trigger.write(AnimationTrigger {
            entity: trigger.target(),
            state: new_animation,
        });
    }
}

pub fn set_unit_idle(
    trigger: Trigger<OnRemove, Moving>,
    mut animation_trigger: EventWriter<AnimationTrigger<UnitAnimation>>,
) {
    animation_trigger.write(AnimationTrigger {
        entity: trigger.target(),
        state: UnitAnimation::Idle,
    });
}

pub fn set_unit_sprite_animation(
    mut command: Commands,
    mut query: Query<(
        Entity,
        &Unit,
        &mut SpriteSheetAnimation,
        &mut Sprite,
        &mut UnitAnimation,
    )>,
    mut animation_changed: EventReader<AnimationTrigger<UnitAnimation>>,
    sprite_sheets: Res<UnitSpriteSheets>,
) {
    for new_animation in animation_changed.read() {
        if let Ok((entity, unit, mut sprite_animation, mut sprite, mut current_animation)) =
            query.get_mut(new_animation.entity)
        {
            if let UnitAnimation::Death = *current_animation {
                continue;
            }

            let animation = sprite_sheets
                .sprite_sheets
                .get(unit.unit_type)
                .animations
                .get(new_animation.state);

            let sound = sprite_sheets
                .sprite_sheets
                .get(unit.unit_type)
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
