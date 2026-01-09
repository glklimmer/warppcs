use bevy::{prelude::*, sprite::Anchor};

use bevy_replicon::prelude::{AppRuleExt, Replicated};
use init_world::StartGamePlugin;
use physics::movement::BoxCollider;
use serde::{Deserialize, Serialize};
use shared::GameScene;
use world::WorldPlugin;

use crate::props::{
    PropsAnimationPlugin,
    trees::{TreeAnimation, pine::PineTreeSpriteSheet},
};

mod props;

pub mod init_world;
pub mod world;

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((StartGamePlugin, WorldPlugin, PropsAnimationPlugin))
            .replicate::<GameScene>()
            .replicate_bundle::<(SceneEnd, Transform)>()
            .add_observer(init_scene_end_sprite);
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
#[require(
    Replicated,
    Transform,
    BoxCollider = scene_end_collider(),
    Sprite,
    Anchor::BOTTOM_CENTER,
)]
pub struct SceneEnd;

fn scene_end_collider() -> BoxCollider {
    BoxCollider {
        dimension: Vec2::new(32., 32.),
        offset: Some(Vec2::new(0., 16.)),
    }
}

fn init_scene_end_sprite(
    trigger: On<Add, SceneEnd>,
    mut scene_end: Query<&mut Sprite>,
    tree_sprite_sheet: Res<PineTreeSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let mut sprite = scene_end.get_mut(trigger.entity)?;

    let bright_sprite_sheet = &tree_sprite_sheet.bright_sprite_sheet;

    let animation = bright_sprite_sheet.animations.get(TreeAnimation::default());
    let texture_atlas = Some(TextureAtlas {
        layout: bright_sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let bright_texture = &bright_sprite_sheet.texture;
    let dim_texture = &tree_sprite_sheet.dim_sprite_sheet.texture;
    let dark_texture = &tree_sprite_sheet.dark_sprite_sheet.texture;

    sprite.image = bright_texture.clone();
    sprite.texture_atlas = texture_atlas.clone();

    let mut entity_commands = commands.entity(trigger.entity);
    entity_commands.insert((animation.clone(), TreeAnimation::default()));

    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(-39., 0., 8.),
        Sprite {
            image: bright_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(-22., 1., 5.),
        Sprite {
            image: dim_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(-14., 0., 3.),
        Sprite {
            image: dim_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(-8., 0., 7.),
        Sprite {
            image: dark_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(8., 2., 6.),
        Sprite {
            image: dim_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(17., 1., 4.),
        Sprite {
            image: dark_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    commands.spawn((
        ChildOf(trigger.entity),
        Transform::from_xyz(25., 2., 1.),
        Sprite {
            image: bright_texture.clone(),
            texture_atlas: texture_atlas.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
    ));
    Ok(())
}
