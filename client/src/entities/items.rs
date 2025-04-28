use bevy::prelude::*;

use bevy::math::bounding::IntersectsVolume;
use shared::{
    BoxCollider, Vec3LayerExt,
    enum_map::EnumIter,
    map::Layers,
    server::players::items::{
        Item, ItemColor, ItemType, MeleeWeapon, ModifierAmount, ProjectileWeapon, WeaponType,
    },
};

use crate::{
    animations::{
        SpriteSheet,
        objects::items::{
            chests::{Chests, ChestsSpriteSheet},
            feet::{Feet, FeetSpriteSheet},
            heads::{Heads, HeadsSpriteSheet},
            weapons::{Weapons, WeaponsSpriteSheet},
        },
    },
    networking::ControlledPlayer,
};

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_item_sprite)
            .add_systems(Update, show_item_info);
    }
}

#[derive(Component, Deref)]
struct ItemInfo(Entity);

pub trait BuildSprite<K> {
    fn sprite_for<T: Into<K>>(&self, kind: T) -> Sprite;
}

impl<K> BuildSprite<K> for SpriteSheet<K>
where
    K: EnumIter,
{
    fn sprite_for<T: Into<K>>(&self, kind: T) -> Sprite {
        let animation = kind.into();
        let sprite_animation = self.animations.get(animation);
        Sprite {
            image: self.texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: self.layout.clone(),
                index: sprite_animation.first_sprite_index,
            }),
            ..Default::default()
        }
    }
}

impl From<WeaponType> for Weapons {
    fn from(wt: WeaponType) -> Self {
        match wt {
            WeaponType::Melee(m) => match m {
                MeleeWeapon::SwordAndShield => Weapons::SwordAndShield,
                MeleeWeapon::Pike => Weapons::Pike,
            },
            WeaponType::Projectile(p) => match p {
                ProjectileWeapon::Bow => Weapons::Bow,
            },
        }
    }
}

impl From<ItemColor> for Chests {
    fn from(c: ItemColor) -> Self {
        match c {
            ItemColor::Brown => Chests::Brown,
            ItemColor::Blue => Chests::Blue,
            ItemColor::Red => Chests::Red,
            ItemColor::Violet => Chests::Violet,
            ItemColor::Green => Chests::Green,
            ItemColor::Beige => Chests::Beige,
        }
    }
}

impl From<ItemColor> for Heads {
    fn from(c: ItemColor) -> Self {
        match c {
            ItemColor::Brown => Heads::Brown,
            ItemColor::Blue => Heads::Blue,
            ItemColor::Red => Heads::Red,
            ItemColor::Violet => Heads::Violet,
            ItemColor::Green => Heads::Green,
            ItemColor::Beige => Heads::Beige,
        }
    }
}

impl From<ItemColor> for Feet {
    fn from(c: ItemColor) -> Self {
        match c {
            ItemColor::Brown => Feet::Brown,
            ItemColor::Blue => Feet::Blue,
            ItemColor::Red => Feet::Red,
            ItemColor::Violet => Feet::Violet,
            ItemColor::Green => Feet::Green,
            ItemColor::Beige => Feet::Beige,
        }
    }
}

pub trait ItemSprite {
    fn sprite(
        &self,
        weapons_sprite_sheet: &Res<WeaponsSpriteSheet>,
        chests_sprite_sheet: &Res<ChestsSpriteSheet>,
        feet_sprite_sheet: &Res<FeetSpriteSheet>,
        heads_sprite_sheet: &Res<HeadsSpriteSheet>,
    ) -> Sprite;
}

impl ItemSprite for Item {
    fn sprite(
        &self,
        weapons_sprite_sheet: &Res<WeaponsSpriteSheet>,
        chests_sprite_sheet: &Res<ChestsSpriteSheet>,
        feet_sprite_sheet: &Res<FeetSpriteSheet>,
        heads_sprite_sheet: &Res<HeadsSpriteSheet>,
    ) -> Sprite {
        match self.item_type {
            ItemType::Weapon(w) => weapons_sprite_sheet.sprite_sheet.sprite_for(w),
            ItemType::Chest => chests_sprite_sheet
                .sprite_sheet
                .sprite_for(self.color.unwrap()),
            ItemType::Head => heads_sprite_sheet
                .sprite_sheet
                .sprite_for(self.color.unwrap()),
            ItemType::Feet => feet_sprite_sheet
                .sprite_sheet
                .sprite_for(self.color.unwrap()),
        }
    }
}

fn init_item_sprite(
    trigger: Trigger<OnAdd, Item>,
    mut commands: Commands,
    mut weapons: Query<(&Item, &Transform)>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    chests_sprite_sheet: Res<ChestsSpriteSheet>,
    feet_sprite_sheet: Res<FeetSpriteSheet>,
    heads_sprite_sheet: Res<HeadsSpriteSheet>,
) {
    let Ok((item, transform)) = weapons.get_mut(trigger.entity()) else {
        return;
    };

    let sprite = item.sprite(
        &weapons_sprite_sheet,
        &chests_sprite_sheet,
        &feet_sprite_sheet,
        &heads_sprite_sheet,
    );

    commands.entity(trigger.entity()).insert(sprite);

    item_info(trigger.entity(), commands.reborrow(), item, transform);
}

fn item_info(entity: Entity, mut commands: Commands, item: &Item, transform: &Transform) {
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

    let mut item_entity = commands.entity(entity);
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
