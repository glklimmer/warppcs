use animations::BuildSprite;
use bevy::prelude::*;

use bevy::{sprite::Anchor, text::TextBounds};

use animations::{
    objects::items::{
        chests::ChestsSpriteSheet, feet::FeetSpriteSheet, heads::HeadsSpriteSheet,
        weapons::WeaponsSpriteSheet,
    },
    ui::item_info::{ItemInfoParts, ItemInfoSpriteSheet},
};
use highlight::Highlighted;
use shared::{
    Vec3LayerExt,
    map::Layers,
    server::players::items::{BaseEffect, Item, ItemType, Modifier, ModifierSign},
};

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(init_item_sprite)
            .add_observer(init_item_info)
            .add_observer(show_item_info)
            .add_observer(hide_item_info);
    }
}

#[derive(Component, Clone)]
pub struct ItemInfo {
    item: Item,
    pub tooltip: Entity,
}

impl ItemInfo {
    pub fn new(item: Item) -> Self {
        Self {
            item,
            tooltip: Entity::PLACEHOLDER,
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
    mut item: Query<&Item>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    chests_sprite_sheet: Res<ChestsSpriteSheet>,
    feet_sprite_sheet: Res<FeetSpriteSheet>,
    heads_sprite_sheet: Res<HeadsSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let item = item.get_mut(trigger.target())?;

    let sprite = item.sprite(
        &weapons_sprite_sheet,
        &chests_sprite_sheet,
        &feet_sprite_sheet,
        &heads_sprite_sheet,
    );

    commands.entity(trigger.target()).insert((
        sprite.clone(),
        ItemInfo {
            item: item.clone(),
            tooltip: trigger.target(),
        },
    ));
    Ok(())
}

fn init_item_info(
    trigger: Trigger<OnAdd, ItemInfo>,
    mut item: Query<&ItemInfo>,
    info: Res<ItemInfoSpriteSheet>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    chests_sprite_sheet: Res<ChestsSpriteSheet>,
    feet_sprite_sheet: Res<FeetSpriteSheet>,
    heads_sprite_sheet: Res<HeadsSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let ItemInfo { item, tooltip: _ } = item.get_mut(trigger.target())?;

    let text_color = Color::srgb_u8(143, 86, 59);

    let info = commands
        .spawn((
            Sprite::from_atlas_image(
                info.sprite_sheet.texture.clone(),
                info.sprite_sheet.texture_atlas(ItemInfoParts::ItemPreview),
            ),
            Vec3::ZERO
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
                        y: 12. + item.base.len() as f32 * 11. + item.modifiers.len() as f32 * 11.,
                    }),
                    image: info.sprite_sheet.texture.clone(),
                    texture_atlas: Some(info.sprite_sheet.texture_atlas(ItemInfoParts::Text)),
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
                Anchor::TOP_CENTER,
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
                    TextBounds::new_horizontal(200.),
                    Transform {
                        translation: Vec3::new(0.5, -58. / 2. - 5., 1.0),
                        scale: Vec3 {
                            x: 0.4,
                            y: 0.4,
                            z: 1.0,
                        },
                        ..Default::default()
                    },
                    Anchor::TOP_CENTER,
                ))
                .with_children(|text_parent| {
                    for BaseEffect { effect, amount } in &item.base {
                        text_parent
                            .spawn((TextSpan::new(format!("{effect}: ")), TextColor(text_color)));

                        text_parent.spawn((
                            TextSpan::new(format!("{amount}\n")),
                            TextColor(Color::BLACK),
                        ));
                    }

                    for Modifier { effect, amount } in &item.modifiers {
                        let amount_text = amount.to_string();
                        let amount_color = match amount.sign() {
                            ModifierSign::Positive => Color::srgb(0., 1., 0.),
                            ModifierSign::Negative => Color::srgb(1., 0., 0.),
                        };

                        text_parent
                            .spawn((TextSpan::new(format!("{effect}: ")), TextColor(text_color)));

                        text_parent.spawn((
                            TextSpan::new(format!("{amount_text}\n")),
                            TextColor(amount_color),
                        ));
                    }
                });
        })
        .id();

    let mut item_entity = commands.entity(trigger.target());
    item_entity.add_child(info);
    item_entity.insert(ItemInfo {
        item: item.clone(),
        tooltip: info,
    });

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
    Ok(())
}

fn show_item_info(
    trigger: Trigger<OnAdd, Highlighted>,
    items: Query<&ItemInfo>,
    mut commands: Commands,
) -> Result {
    if let Ok(info) = items.get(trigger.target()) {
        let mut entity = commands.get_entity(info.tooltip)?;
        entity.try_insert(Visibility::Visible);
    };
    Ok(())
}

fn hide_item_info(
    trigger: Trigger<OnRemove, Highlighted>,
    items: Query<&ItemInfo>,
    mut commands: Commands,
) -> Result {
    if let Ok(info) = items.get(trigger.target()) {
        let mut entity = commands.get_entity(info.tooltip)?;
        entity.try_insert(Visibility::Hidden);
    };
    Ok(())
}
