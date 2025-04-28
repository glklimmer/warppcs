use bevy::{prelude::*, sprite::Anchor, text::TextBounds};

use shared::{
    Vec3LayerExt,
    enum_map::EnumIter,
    map::Layers,
    server::players::items::{
        Item, ItemColor, ItemType, MeleeWeapon, ModifierAmount, ProjectileWeapon, WeaponType,
    },
};

use crate::animations::{
    SpriteSheet,
    objects::items::{
        chests::{Chests, ChestsSpriteSheet},
        feet::{Feet, FeetSpriteSheet},
        heads::{Heads, HeadsSpriteSheet},
        weapons::{Weapons, WeaponsSpriteSheet},
    },
    ui::item_info::ItemInfoSpriteSheet,
};

use super::highlight::Highlighted;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_item_sprite)
            .add_observer(init_item_info)
            .add_observer(show_item_info)
            .add_observer(hide_item_info);
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
    mut item: Query<&Item>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    chests_sprite_sheet: Res<ChestsSpriteSheet>,
    feet_sprite_sheet: Res<FeetSpriteSheet>,
    heads_sprite_sheet: Res<HeadsSpriteSheet>,
) {
    let Ok(item) = item.get_mut(trigger.entity()) else {
        return;
    };

    let sprite = item.sprite(
        &weapons_sprite_sheet,
        &chests_sprite_sheet,
        &feet_sprite_sheet,
        &heads_sprite_sheet,
    );

    commands.entity(trigger.entity()).insert(sprite.clone());
}

fn init_item_info(
    trigger: Trigger<OnAdd, Item>,
    mut commands: Commands,
    mut item: Query<(&Item, &Transform)>,
    info: Res<ItemInfoSpriteSheet>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    chests_sprite_sheet: Res<ChestsSpriteSheet>,
    feet_sprite_sheet: Res<FeetSpriteSheet>,
    heads_sprite_sheet: Res<HeadsSpriteSheet>,
) {
    let Ok((item, transform)) = item.get_mut(trigger.entity()) else {
        return;
    };

    let text_color = Color::srgb_u8(143, 86, 59);

    let info = commands
        .spawn((
            Sprite {
                texture_atlas: Some(TextureAtlas {
                    layout: info.layout.clone(),
                    index: 0,
                }),
                image: info.texture.clone(),
                ..Default::default()
            },
            transform
                .translation
                .offset_x(48.)
                .offset_y(20.)
                .with_layer(Layers::Item)
                .with_scale(Vec3 {
                    x: 1. / 2.,
                    y: 1. / 2.,
                    z: 1.,
                }),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Background for text
            parent.spawn((
                Sprite {
                    custom_size: Some(Vec2 {
                        x: 100.,
                        y: 12. + item.modifiers.len() as f32 * 11.,
                    }),
                    anchor: Anchor::TopCenter,
                    image: info.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: info.layout.clone(),
                        index: 1,
                    }),
                    image_mode: SpriteImageMode::Sliced(TextureSlicer {
                        border: BorderRect {
                            left: 2.,
                            right: 2.,
                            top: 0.,
                            bottom: 2.,
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                Transform::from_xyz(0., -58. / 2., 0.),
            ));

            // Text
            parent
                .spawn((
                    Text2d::new(""),
                    TextFont {
                        font_size: 124.0,
                        ..default()
                    },
                    TextLayout::new_with_justify(JustifyText::Left).with_no_wrap(),
                    TextColor(text_color),
                    TextBounds::new_horizontal(92.),
                    Anchor::TopCenter,
                    Transform {
                        translation: Vec3::new(0.5, -58. / 2. - 5., 1.0),
                        scale: Vec3 {
                            x: 0.4,
                            y: 0.4,
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
                            ModifierAmount::Base(_) => Color::BLACK,
                            ModifierAmount::Multiplier(amount) => {
                                if *amount > 0 {
                                    Color::srgb(0., 1., 0.)
                                } else if *amount < 0 {
                                    Color::srgb(1., 0., 0.)
                                } else {
                                    text_color
                                }
                            }
                        };

                        // Effect label
                        text_parent
                            .spawn((TextSpan::new(format!("{effect}: ")), TextColor(text_color)));

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

    let item_sprite = commands
        .spawn((
            item.sprite(
                &weapons_sprite_sheet,
                &chests_sprite_sheet,
                &feet_sprite_sheet,
                &heads_sprite_sheet,
            ),
            Transform::from_xyz(0., 0., 2.).with_scale(Vec3 {
                x: 2.,
                y: 2.,
                z: 1.,
            }),
        ))
        .id();

    let mut info = commands.entity(info);
    info.add_child(item_sprite);
}

fn show_item_info(
    trigger: Trigger<OnAdd, Highlighted>,
    mut commands: Commands,
    items: Query<&ItemInfo>,
) {
    let Ok(info) = items.get(trigger.entity()) else {
        return;
    };
    let Some(mut entity) = commands.get_entity(**info) else {
        return;
    };
    entity.try_insert(Visibility::Visible);
}

fn hide_item_info(
    trigger: Trigger<OnRemove, Highlighted>,
    mut commands: Commands,
    items: Query<&ItemInfo>,
) {
    let Ok(info) = items.get(trigger.entity()) else {
        return;
    };
    let Some(mut entity) = commands.get_entity(**info) else {
        return;
    };
    entity.try_insert(Visibility::Hidden);
}
