use bevy::prelude::*;

use bevy::{asset::RenderAssetUsages, math::bounding::IntersectsVolume};
use image::{GenericImage, GenericImageView, Rgba};
use shared::server::physics::attachment::AttachedTo;
use shared::{BoxCollider, Player, server::players::interaction::Interactable};
use std::cmp::Ordering;

use crate::networking::ControlledPlayer;

#[derive(Component)]
struct Highlightable {
    outline_color: Color,
}

impl Default for Highlightable {
    fn default() -> Self {
        Self {
            outline_color: Color::WHITE,
        }
    }
}

#[derive(Component, Default)]
pub struct Highlighted;

#[derive(Component, Deref)]
struct OriginalSprite(Handle<Image>);

pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_highlightable)
            .add_observer(outline_sprite)
            .add_observer(restore_original_sprite)
            .add_observer(remove_highlightable)
            .add_systems(PostUpdate, check_highlight);
    }
}

#[allow(clippy::type_complexity)]
fn check_highlight(
    mut commands: Commands,
    outline: Query<
        (
            Entity,
            &Transform,
            &BoxCollider,
            Option<&Interactable>,
            Option<&AttachedTo>,
        ),
        With<Highlightable>,
    >,
    player: Query<(&Transform, &BoxCollider), With<ControlledPlayer>>,
) {
    let Ok((player_transform, player_collider)) = player.single() else {
        return;
    };

    let player_bounds = player_collider.at(player_transform);

    let priority_interaction = outline
        .iter()
        .filter(|(_, transform, collider, ..)| collider.at(transform).intersects(&player_bounds))
        .max_by(
            |(_, a_transform, .., interactable_a, _), (_, b_transform, .., interactable_b, _)| {
                match (interactable_a, interactable_b) {
                    (Some(a), Some(b)) => {
                        let priority_a = a.kind as i32;
                        let priority_b = b.kind as i32;

                        if priority_a != priority_b {
                            return priority_a.cmp(&priority_b);
                        }

                        let distance_a = player_transform
                            .translation
                            .distance(a_transform.translation);
                        let distance_b = player_transform
                            .translation
                            .distance(b_transform.translation);
                        distance_b.total_cmp(&distance_a)
                    }
                    (None, None) => {
                        let distance_a = player_transform
                            .translation
                            .distance(a_transform.translation);
                        let distance_b = player_transform
                            .translation
                            .distance(b_transform.translation);
                        distance_b.total_cmp(&distance_a)
                    }
                    (None, Some(_)) => Ordering::Less,
                    (Some(_), None) => Ordering::Greater,
                }
            },
        )
        .map(|(entity, ..)| entity);

    let Some(priority_entity) = priority_interaction else {
        for (entity, ..) in outline.iter() {
            commands.entity(entity).try_remove::<Highlighted>();
        }
        return;
    };

    for (entity, .., maybe_attached_to) in outline.iter() {
        if entity.eq(&priority_entity) && maybe_attached_to.is_none() {
            commands.entity(entity).insert(Highlighted);
        } else {
            commands.entity(entity).try_remove::<Highlighted>();
        }
    }
}

fn init_highlightable(
    trigger: Trigger<OnAdd, Interactable>,
    mut commands: Commands,
    controlled_player: Query<Entity, With<ControlledPlayer>>,
    interactable: Query<&Interactable, Without<Player>>,
) {
    let Ok(interactable) = interactable.get(trigger.target()) else {
        return;
    };

    let controller_player = controlled_player.single().unwrap();

    if let Some(owner) = interactable.restricted_to
        && owner != controller_player
    {
        return;
    }
    commands
        .entity(trigger.target())
        .try_insert(Highlightable::default());
}

fn remove_highlightable(trigger: Trigger<OnRemove, Interactable>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .try_remove::<Highlightable>()
        .try_remove::<Highlighted>();
}

fn restore_original_sprite(
    trigger: Trigger<OnRemove, Highlighted>,
    mut commands: Commands,
    mut query: Query<(&mut Sprite, &OriginalSprite)>,
) {
    if let Ok((mut sprite, original_sprite)) = query.get_mut(trigger.target()) {
        sprite.image = (**original_sprite).clone();
        commands
            .entity(trigger.target())
            .try_remove::<OriginalSprite>();
    }
}

fn outline_sprite(
    trigger: Trigger<OnAdd, Highlighted>,
    mut commands: Commands,
    mut query: Query<(&mut Sprite, &Highlightable)>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Ok((mut sprite, highlightable)) = query.get_mut(trigger.target()) {
        let outline_color = highlightable.outline_color;

        commands
            .entity(trigger.target())
            .insert(OriginalSprite(sprite.image.clone()));

        let maybe_image = images.get(sprite.image.id());

        if let Some(texture) = maybe_image {
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
                    outlined_image.put_pixel(x, y, Rgba(outline_color.to_srgba().to_u8_array()));
                }
            }

            let outline_image =
                Image::from_dynamic(outlined_image, true, RenderAssetUsages::RENDER_WORLD);
            sprite.image = images.add(outline_image);
        }
    }
}
