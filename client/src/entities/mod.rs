use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use highlight::HighlightPlugin;
use shared::{
    BoxCollider, Vec3LayerExt,
    map::Layers,
    server::players::items::{
        Item, ItemType, MeleeWeapon, ModifierAmount, ProjectileWeapon, WeaponType,
    },
};
use spawn::SpawnPlugin;

use crate::{
    animations::objects::items::weapons::{Weapons, WeaponsSpriteSheet},
    networking::ControlledPlayer,
};

mod spawn;

pub mod highlight;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpawnPlugin)
            .add_plugins(HighlightPlugin)
            .add_observer(init_weapon_sprite)
            .add_systems(Update, show_item_info);
    }
}

#[derive(Component, Deref)]
struct ItemInfo(Entity);

fn init_weapon_sprite(
    trigger: Trigger<OnAdd, Item>,
    mut commands: Commands,
    mut weapons: Query<(&mut Sprite, &Item, &Transform)>,
    sprite_sheets: Res<WeaponsSpriteSheet>,
) {
    let Ok((mut sprite, item, transform)) = weapons.get_mut(trigger.entity()) else {
        return;
    };
    let ItemType::Weapon(weapon) = item.item_type else {
        return;
    };

    let sprite_sheet = &sprite_sheets.sprite_sheet;
    sprite.image = sprite_sheet.texture.clone();

    let weapon = match weapon {
        WeaponType::Melee(use_weapon) => match use_weapon {
            MeleeWeapon::SwordAndShield => Weapons::SwordAndShield,
            MeleeWeapon::Pike => Weapons::Pike,
        },
        WeaponType::Projectile(projectile_weapon) => match projectile_weapon {
            ProjectileWeapon::Bow => Weapons::Bow,
        },
    };

    let animation = sprite_sheet.animations.get(weapon);
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprite_sheet.layout.clone(),
        index: animation.first_sprite_index,
    });

    let info = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.106, 0.118, 0.122),
                custom_size: Some(Vec2::new(47.0, 33.0)),
                ..Default::default()
            },
            transform
                .translation
                .offset_x(48.)
                .offset_y(20.)
                .with_layer(Layers::Item),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Border
            parent.spawn((
                Sprite {
                    color: item.rarity.color(),
                    custom_size: Some(Vec2::new(50.0, 36.0)),
                    ..Default::default()
                },
                Transform::from_xyz(0., 0., -1.),
            ));

            // Text
            parent
                .spawn((
                    Text2d::new(""),
                    TextFont {
                        font_size: 124.0,
                        ..default()
                    },
                    TextLayout::new_with_justify(JustifyText::Left),
                    TextColor(Color::WHITE),
                    Transform {
                        translation: Vec3::new(0.5, 0.0, 1.0),
                        scale: Vec3 {
                            x: 0.2,
                            y: 0.2,
                            z: 1.0,
                        },
                        ..Default::default()
                    },
                ))
                .with_children(|text_parent| {
                    for modifier in &item.modifiers {
                        let effect = &modifier.effect;
                        let modifier = &modifier.amount;
                        let amount_str = &modifier;

                        let amount_color = match modifier {
                            ModifierAmount::Base(_) => Color::WHITE,
                            ModifierAmount::Multiplier(amount) => {
                                if *amount > 0 {
                                    Color::srgb(0., 1., 0.)
                                } else if *amount < 0 {
                                    Color::srgb(1., 0., 0.)
                                } else {
                                    Color::WHITE
                                }
                            }
                        };

                        // Effect label
                        text_parent.spawn((
                            TextSpan::new(format!("{effect}: ")),
                            TextColor(Color::WHITE),
                        ));

                        // Amount with color
                        text_parent.spawn((
                            TextSpan::new(format!("{amount_str}\n")),
                            TextColor(amount_color),
                        ));
                    }
                });
        })
        .id();

    let mut item_entity = commands.entity(trigger.entity());
    item_entity.add_child(info);
    item_entity.insert(ItemInfo(info));
}

fn show_item_info(
    mut commands: Commands,
    player: Query<(&Transform, &BoxCollider), With<ControlledPlayer>>,
    items: Query<(&ItemInfo, &Transform, &BoxCollider)>,
) {
    let Ok((player_transform, player_collider)) = player.get_single() else {
        return;
    };

    let player_bounds = player_collider.at(player_transform);

    for (info, transform, collider) in items.iter() {
        let mut entity = commands.entity(**info);

        if player_bounds.intersects(&collider.at(transform)) {
            entity.insert(Visibility::Visible);
        } else {
            entity.insert(Visibility::Hidden);
        }
    }
}
