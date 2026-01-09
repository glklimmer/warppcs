use bevy::prelude::*;

use animations::{AnimationSpriteSheet, SpriteVariants, SpriteVariantsAssetsExt, anim};
use highlight::Highlighted;
use physics::attachment::AttachedTo;
use serde::{Deserialize, Serialize};
use shared::{Player, enum_map::*};

use crate::flag::{Flag, FlagDestroyed};

pub(crate) struct FlagAnimationPlugin;

impl Plugin for FlagAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlagSpriteSheet>()
            .add_observer(on_flag_destroyed)
            .add_observer(init_flag_sprite)
            .add_systems(Update, update_flag_visibility);
    }
}

const ATLAS_COLUMNS: usize = 8;

#[derive(
    Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default, Deserialize, Serialize,
)]
enum FlagAnimation {
    #[default]
    Wave,
    Destroyed,
}

#[derive(Resource)]
struct FlagSpriteSheet {
    pub sprite_sheet: AnimationSpriteSheet<FlagAnimation, SpriteVariants>,
}

impl FromWorld for FlagSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture = asset_server.load("sprites/objects/flag.png");

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(24, 24),
            ATLAS_COLUMNS as u32,
            2,
            Some(UVec2::splat(1)),
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            FlagAnimation::Wave => anim!(0, 7),
            FlagAnimation::Destroyed => anim!(1, 7),
        });

        let animations_sound = EnumMap::new(|c| match c {
            FlagAnimation::Wave => None,
            FlagAnimation::Destroyed => None,
        });

        FlagSpriteSheet {
            sprite_sheet: AnimationSpriteSheet::new(
                world,
                texture,
                layout,
                animations,
                animations_sound,
            ),
        }
    }
}

fn init_flag_sprite(
    trigger: On<Add, Flag>,
    mut flag: Query<(&mut Sprite, &Flag)>,
    flag_sprite_sheet: Res<FlagSpriteSheet>,
    variants: Res<Assets<SpriteVariants>>,
    mut commands: Commands,
) -> Result {
    let (mut sprite, flag) = flag.get_mut(trigger.entity)?;

    let sprite_sheet = &flag_sprite_sheet.sprite_sheet;
    let handle = &sprite_sheet.texture;
    let sprite_variants = variants.get_variant(handle)?;
    let animation = sprite_sheet.animations.get(FlagAnimation::default());

    sprite.image = sprite_variants.variants.get(flag.color).clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((animation.clone(), FlagAnimation::default()));
    Ok(())
}

// TODO: Change to Observer after Replcion 0.34 update
fn update_flag_visibility(
    flag: Query<(Entity, &AttachedTo), Changed<AttachedTo>>,
    player: Query<&Player>,
    mut commands: Commands,
) {
    for (flag, attachted_to) in flag.iter() {
        if player.get(**attachted_to).is_ok() {
            commands.entity(flag).insert(Visibility::Visible);
        } else {
            commands.entity(flag).insert(Visibility::Hidden);
        }
    }
}

fn on_flag_destroyed(
    trigger: On<Add, FlagDestroyed>,
    mut query: Query<&mut Sprite>,
    flag_sprite_sheet: Res<FlagSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.entity;
    let mut sprite = query.get_mut(entity)?;

    let animation = flag_sprite_sheet
        .sprite_sheet
        .animations
        .get(FlagAnimation::Destroyed);

    commands
        .entity(entity)
        .insert(animation.clone())
        .remove::<Highlighted>();

    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = animation.first_sprite_index;
    }
    Ok(())
}
