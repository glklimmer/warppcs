use bevy::prelude::*;

use animations::StaticSpriteSheet;
use shared::enum_map::*;

use crate::ProjectileType;

pub(crate) struct ProjectileAnimationPlugin;

impl Plugin for ProjectileAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectileSpriteSheet>()
            .add_observer(init_projectile_sprite);
    }
}

#[derive(Debug, Clone, Copy, Mappable)]
enum Projectiles {
    Arrow,
}

#[derive(Resource)]
struct ProjectileSpriteSheet {
    sprite_sheet: StaticSpriteSheet<Projectiles>,
}

impl FromWorld for ProjectileSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/humans/HumansProjectiles.png");
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2 { x: 16, y: 16 },
            5,
            3,
            None,
            None,
        ));

        let parts = EnumMap::new(|c| match c {
            Projectiles::Arrow => 1,
        });

        Self {
            sprite_sheet: StaticSpriteSheet::new(world, texture, layout, parts),
        }
    }
}

fn init_projectile_sprite(
    trigger: On<Add, ProjectileType>,
    mut projectile: Query<(&mut Sprite, &ProjectileType)>,
    projectiles: Res<ProjectileSpriteSheet>,
) -> Result {
    let (mut sprite, projectile_type) = projectile.get_mut(trigger.entity)?;

    let texture = match projectile_type {
        ProjectileType::Arrow => projectiles.sprite_sheet.texture_atlas(Projectiles::Arrow),
    };
    sprite.texture_atlas = Some(texture);
    sprite.image = projectiles.sprite_sheet.texture.clone();
    Ok(())
}
