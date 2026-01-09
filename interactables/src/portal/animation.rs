use bevy::prelude::*;

use shared::enum_map::*;

use animations::{AnimationSpriteSheet, anim};

use crate::portal::Portal;

pub(crate) struct PortalAnimationPlugin;

impl Plugin for PortalAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PortalSpriteSheet>()
            .add_observer(init_portal_sprite);
    }
}

const ATLAS_COLUMNS: usize = 12;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Mappable, Default)]
enum PortalAnimation {
    #[default]
    Swirle,
}

#[derive(Resource)]
struct PortalSpriteSheet {
    sprite_sheet: AnimationSpriteSheet<PortalAnimation, Image>,
}

impl FromWorld for PortalSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture = asset_server.load("sprites/objects/portal.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(32, 32),
            ATLAS_COLUMNS as u32,
            1,
            None,
            None,
        ));

        let animations = EnumMap::new(|c| match c {
            PortalAnimation::Swirle => anim!(0, 11),
        });

        let animations_sound = EnumMap::new(|c| match c {
            PortalAnimation::Swirle => None,
        });

        PortalSpriteSheet {
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

fn init_portal_sprite(
    trigger: On<Add, Portal>,
    mut portal: Query<&mut Sprite>,
    portal_sprite_sheet: Res<PortalSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let mut sprite = portal.get_mut(trigger.entity)?;

    let sprite_sheet = &portal_sprite_sheet.sprite_sheet;
    let animation = sprite_sheet.animations.get(PortalAnimation::default());

    sprite.image = sprite_sheet.texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let mut commands = commands.entity(trigger.entity);
    commands.insert((animation.clone(), PortalAnimation::default()));
    Ok(())
}
