use bevy::{
    asset::RenderAssetUsages,
    ecs::{component::ComponentId, world::DeferredWorld},
    math::bounding::IntersectsVolume,
    prelude::*,
};
use image::{GenericImage, GenericImageView, Rgba};
use shared::{BoxCollider, GameState};

use crate::networking::ControlledPlayer;

fn on_remove_highlighted(mut world: DeferredWorld, entity: Entity, id: ComponentId) {
    let mut entity_mut = world.entity_mut(entity);
    let value = entity_mut
        .get::<Highlighted>()
        .unwrap()
        .original_handle
        .clone();

    let mut sprite = entity_mut.get_mut::<Sprite>().unwrap();
    sprite.image = value
}

#[derive(Component)]
#[component(on_remove = on_remove_highlighted)]
pub struct Highlighted {
    original_handle: Handle<Image>,
}

#[derive(Component)]
pub struct Highlightable {
    pub outline_color: Color,
}

impl Default for Highlightable {
    fn default() -> Self {
        Self {
            outline_color: Color::WHITE,
        }
    }
}

pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (highlight_entity, check_highlight).run_if(in_state(GameState::GameSession)),
        );
    }
}

fn highlight_entity(
    mut sprites: Query<&mut Sprite, Changed<Highlighted>>,
    mut images: ResMut<Assets<Image>>,
) {
    for mut texture_handle in sprites.iter_mut() {
        if let Some(texture) = images.get_mut(texture_handle.image.id()) {
            let width = texture.width() as u32;
            let height = texture.height() as u32;
            let dynamic_image = texture.clone().try_into_dynamic().unwrap();
            let mut outlined_image = dynamic_image.clone();
            for (x, y, p) in dynamic_image.pixels() {
                if x == 0 || y == 0 || x == width - 1 || y == height - 1 || p.0[3] != 0 {
                    continue;
                }
                let current = dynamic_image.get_pixel(x, y)[3];
                let left = dynamic_image.get_pixel(x - 1, y)[3];
                let right = dynamic_image.get_pixel(x + 1, y)[3];
                let up = dynamic_image.get_pixel(x, y - 1)[3];
                let down = dynamic_image.get_pixel(x, y + 1)[3];
                if current != left || current != right || current != up || current != down {
                    outlined_image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
            }

            let outline_image =
                Image::from_dynamic(outlined_image, true, RenderAssetUsages::RENDER_WORLD);
            texture_handle.image = images.add(outline_image);
        }
    }
}

fn check_highlight(
    mut commands: Commands,
    mut outline: Query<
        (
            Entity,
            &Transform,
            &BoxCollider,
            &Sprite,
            Option<&mut Highlighted>,
        ),
        With<Highlightable>,
    >,
    player: Query<(&Transform, &BoxCollider), With<ControlledPlayer>>,
) {
    let (player_transform, player_collider) = player.get_single().unwrap();
    let player_bounds = player_collider.at(player_transform);

    for (entity, transform, box_collider, sprite, highlighted) in outline.iter_mut() {
        let bounds = box_collider.at(transform);
        let intersected = bounds.intersects(&player_bounds);
        match highlighted {
            Some(_) => {
                if !intersected {
                    commands.entity(entity).remove::<Highlighted>();
                }
            }
            None => {
                if intersected {
                    commands.entity(entity).insert(Highlighted {
                        original_handle: sprite.image.clone(),
                    });
                }
            }
        }
    }
}
