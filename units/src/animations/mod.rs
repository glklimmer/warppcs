use bevy::prelude::*;

use animations::{
    AnimationSpriteSheet, AnimationTrigger, PlayOnce, SpriteSheetAnimation, SpriteVariants,
    SpriteVariantsAssetsExt, sound::AnimationSound,
};
use bandits::bandit::bandit;
use bevy_replicon::client::ClientSystems;
use health::Health;
use humans::{
    archer::archer, commander::commander, pikeman::pikeman, shieldwarrior::shieldwarrior,
};
use physics::movement::Moving;
use shared::{AnimationChange, AnimationChangeEvent, enum_map::*, server::entities::UnitAnimation};

use crate::{Unit, UnitType};

mod bandits;
mod humans;

pub(crate) struct UnitAnimationPlugin;

impl Plugin for UnitAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnitSpriteSheets>()
            .add_message::<AnimationTrigger<UnitAnimation>>()
            .add_observer(init_unit_sprite)
            .add_observer(set_unit_walking)
            .add_observer(set_unit_idle)
            .add_observer(set_unit_after_play_once)
            .add_systems(
                PreUpdate,
                (trigger_unit_animation).after(ClientSystems::Receive),
            )
            .add_systems(Update, set_unit_sprite_animation);
    }
}

const DIRT_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/dirt_footsteps";
const GRASS_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/grass_footsteps";
// const SNOW_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/snow_footsteps";
// const STONE_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/stone_footsteps";
// const WATER_FOOTSTEPS_SOUND_PATH: &str = "animation_sound/footsteps/water_footsteps";
// const HORSE_SOUND_PATH: &str = "animation_sound/horse";

#[derive(Resource)]
struct UnitSpriteSheets {
    sprite_sheets: EnumMap<UnitType, AnimationSpriteSheet<UnitAnimation, SpriteVariants>>,
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

fn init_unit_sprite(
    trigger: On<Add, Unit>,
    mut units: Query<(&mut Sprite, &Unit, Option<&Health>)>,
    sprite_sheets: Res<UnitSpriteSheets>,
    variants: Res<Assets<SpriteVariants>>,
    mut commands: Commands,
) -> Result {
    let (mut sprite, unit, maybe_health) = units.get_mut(trigger.entity)?;

    let sprite_sheet = &sprite_sheets.sprite_sheets.get(unit.unit_type);
    let handle = &sprite_sheet.texture;
    let sprite_variants = variants.get_variant(handle)?;
    let animation = match maybe_health {
        Some(_) => UnitAnimation::Idle,
        None => UnitAnimation::Death,
    };
    let sprite_sheet_animation = sprite_sheet.animations.get(animation);

    sprite.image = sprite_variants.variants.get(unit.color).clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: sprite_sheet_animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((sprite_sheet_animation.clone(), animation));
    Ok(())
}

fn trigger_unit_animation(
    mut network_events: MessageReader<AnimationChangeEvent>,
    mut animation_trigger: MessageWriter<AnimationTrigger<UnitAnimation>>,
    mut commands: Commands,
) {
    for event in network_events.read() {
        let new_animation = match &event.change {
            AnimationChange::Idle => UnitAnimation::Idle,
            AnimationChange::Attack => UnitAnimation::Attack,
            AnimationChange::Hit(_) => UnitAnimation::Hit,
            AnimationChange::Death => UnitAnimation::Death,
            AnimationChange::KnockOut => UnitAnimation::Death,
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

fn set_unit_walking(
    trigger: On<Add, Moving>,
    is_unit: Query<Entity, With<Unit>>,
    mut animation_trigger: MessageWriter<AnimationTrigger<UnitAnimation>>,
) {
    if is_unit.get(trigger.entity).is_ok() {
        animation_trigger.write(AnimationTrigger {
            entity: trigger.entity,
            state: UnitAnimation::Walk,
        });
    }
}

fn set_unit_after_play_once(
    trigger: On<Remove, PlayOnce>,
    mut animation_trigger: MessageWriter<AnimationTrigger<UnitAnimation>>,
    unit_animation: Query<&UnitAnimation>,
    mut commands: Commands,
) {
    if let Ok(animation) = unit_animation.get(trigger.entity) {
        let mut entity = commands.entity(trigger.entity);
        if let UnitAnimation::Death = animation {
            entity.remove::<SpriteSheetAnimation>();
            return;
        }

        let new_animation = match animation {
            UnitAnimation::Attack | UnitAnimation::Hit => UnitAnimation::Idle,
            _ => *animation,
        };

        animation_trigger.write(AnimationTrigger {
            entity: trigger.entity,
            state: new_animation,
        });
    }
}

fn set_unit_idle(
    trigger: On<Remove, Moving>,
    is_unit: Query<Entity, With<Unit>>,
    mut animation_trigger: MessageWriter<AnimationTrigger<UnitAnimation>>,
) {
    if is_unit.get(trigger.entity).is_ok() {
        animation_trigger.write(AnimationTrigger {
            entity: trigger.entity,
            state: UnitAnimation::Idle,
        });
    }
}

fn set_unit_sprite_animation(
    mut query: Query<(
        Entity,
        &Unit,
        &mut SpriteSheetAnimation,
        &mut Sprite,
        &mut UnitAnimation,
    )>,
    mut animation_changed: MessageReader<AnimationTrigger<UnitAnimation>>,
    sprite_sheets: Res<UnitSpriteSheets>,
    mut command: Commands,
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
