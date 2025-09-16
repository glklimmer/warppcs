use bevy::prelude::*;

use bevy::{
    asset::RenderAssetUsages,
    ecs::{component::HookContext, world::DeferredWorld},
    math::bounding::IntersectsVolume,
};
use image::{GenericImage, GenericImageView, Rgba};
use shared::{
    BoxCollider, Player,
    server::{physics::attachment::AttachedTo, players::interaction::Interactable},
};
use std::cmp::Ordering;

use crate::networking::ControlledPlayer;

fn on_remove_highlighted(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
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

#[derive(Component)]
#[component(on_remove = on_remove_highlighted)]
pub struct Highlighted {
    pub original_handle: Handle<Image>,
}

pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(remove_highlightable_on_attached)
            .add_observer(add_highlightable_on_attached)
            .add_observer(init_highlightable)
            .add_observer(remove_highlightable)
            .add_systems(PostUpdate, (highlight_entity, check_highlight));
    }
}

fn highlight_entity(
    mut sprites: Query<(&mut Sprite, &Highlightable), Changed<Highlighted>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (mut sprite, highlightable) in sprites.iter_mut() {
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
                    outlined_image.put_pixel(
                        x,
                        y,
                        Rgba(highlightable.outline_color.to_srgba().to_u8_array()),
                    );
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
    mut outline: Query<
        (
            Entity,
            &Transform,
            &BoxCollider,
            &Sprite,
            Option<&mut Highlighted>,
            Option<&Interactable>,
        ),
        With<Highlightable>,
    >,
    player: Query<(&Transform, &BoxCollider), With<ControlledPlayer>>,
) {
    let Ok((player_transform, player_collider)) = player.single() else {
        return;
    };

    let player_bounds = player_collider.at(player_transform);

    let candidate_entity = outline
        .iter()
        .filter(|(_, transform, collider, ..)| collider.at(transform).intersects(&player_bounds))
        .max_by(
            |(_, a_transform, .., interactable_a), (_, b_transform, .., interactable_b)| match (
                interactable_a,
                interactable_b,
            ) {
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
            },
        )
        .map(|(entity, ..)| entity);

    for (entity, _, _, sprite, maybe_highlight, _) in outline.iter_mut() {
        if Some(entity) == candidate_entity {
            if maybe_highlight.is_none() {
                commands.entity(entity).insert(Highlighted {
                    original_handle: sprite.image.clone(),
                });
            }
        } else if maybe_highlight.is_some() {
            commands.entity(entity).remove::<Highlighted>();
        }
    }
}

fn init_highlightable(
    trigger: Trigger<OnAdd, Interactable>,
    mut commands: Commands,
    controlled_player: Query<Entity, With<ControlledPlayer>>,
    interactable: Query<(&Interactable, Option<&AttachedTo>), Without<Player>>,
) {
    let Ok((interactable, maybe_attached)) = interactable.get(trigger.target()) else {
        return;
    };

    if maybe_attached.is_some() {
        return;
    }

    let controller_player = controlled_player.single().unwrap();

    if let Some(owner) = interactable.restricted_to {
        if owner != controller_player {
            return;
        }
    }
    commands
        .entity(trigger.target())
        .insert(Highlightable::default());
}

fn remove_highlightable(trigger: Trigger<OnRemove, Interactable>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .try_remove::<Highlightable>()
        .try_remove::<Highlighted>();
}

fn remove_highlightable_on_attached(trigger: Trigger<OnAdd, AttachedTo>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .try_remove::<Highlightable>()
        .try_remove::<Highlighted>();
}

fn add_highlightable_on_attached(
    trigger: Trigger<OnRemove, AttachedTo>,
    mut commands: Commands,
    query: Query<&Interactable>,
) {
    if query.get(trigger.target()).is_err() {
        return;
    }

    commands
        .entity(trigger.target())
        .insert(Highlightable::default());
}
