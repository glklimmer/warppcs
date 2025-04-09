use bevy::{
    asset::RenderAssetUsages,
    ecs::{component::ComponentId, world::DeferredWorld},
    math::bounding::IntersectsVolume,
    prelude::*,
};
use image::{GenericImage, GenericImageView, Rgba};
use shared::{
    BoxCollider, Faction, Highlightable, Owner,
    server::{physics::attachment::AttachedTo, players::interaction::Interactable},
};

use crate::networking::ControlledPlayer;

fn on_remove_highlighted(mut world: DeferredWorld, entity: Entity, _id: ComponentId) {
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
    pub original_handle: Handle<Image>,
}

pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (highlight_entity, check_highlight));
    }
}

fn highlight_entity(
    mut sprites: Query<&mut Sprite, Changed<Highlighted>>,
    mut images: ResMut<Assets<Image>>,
) {
    for mut sprite in sprites.iter_mut() {
        if let Some(texture) = images.get_mut(sprite.image.id()) {
            let width = texture.width();
            let height = texture.height();
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
            sprite.image = images.add(outline_image);
        }
    }
}

#[allow(clippy::type_complexity)]
fn check_highlight(
    mut commands: Commands,
    mut outline: Query<(
        Entity,
        &Transform,
        &BoxCollider,
        &Sprite,
        &Interactable,
        Option<&mut Highlighted>,
        Option<&AttachedTo>,
    )>,
    player: Query<(Entity, &Transform, &BoxCollider), With<ControlledPlayer>>,
) {
    let Ok((player_entity, player_transform, player_collider)) = player.get_single() else {
        return;
    };

    let player_bounds = player_collider.at(player_transform);

    for (entity, transform, box_collider, sprite, interactable, highlighted, attached_to) in
        outline.iter_mut()
    {
        let bounds = box_collider.at(transform);
        let intersected = bounds.intersects(&player_bounds);
        match highlighted {
            Some(_) => {
                if !intersected || attached_to.is_some() {
                    commands.entity(entity).remove::<Highlighted>();
                }
            }
            None => {
                if intersected && attached_to.is_none() {
                    if let Some(owner) = interactable.restricted_to {
                        match owner.0 {
                            Faction::Player(entity) => {
                                if entity != player_entity {
                                    continue;
                                }
                            }
                            Faction::Bandits => return,
                        }
                    }
                    commands.entity(entity).insert(Highlighted {
                        original_handle: sprite.image.clone(),
                    });
                }
            }
        }
    }
}
