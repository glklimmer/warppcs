use bevy::prelude::*;
use shared::{
    Vec3LayerExt,
    map::Layers,
    server::{
        buildings::item_assignment::{ItemAssignment, OpenItemAssignment},
        players::items::ItemType,
    },
};

use crate::animations::objects::items::{
    chests::ChestsSpriteSheet, feet::FeetSpriteSheet, heads::HeadsSpriteSheet,
    weapons::WeaponsSpriteSheet,
};

pub struct ItemAssignmentPlugin;

impl Plugin for ItemAssignmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(open_assignment_dialog);
    }
}

fn open_assignment_dialog(
    trigger: Trigger<OpenItemAssignment>,
    mut commands: Commands,
    building: Query<(&Transform, &ItemAssignment)>,
    asset_server: Res<AssetServer>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    chests_sprite_sheet: Res<ChestsSpriteSheet>,
    heads_sprite_sheet: Res<HeadsSpriteSheet>,
    feet_sprite_sheet: Res<FeetSpriteSheet>,
) {
    let (transform, assignment) = building.get(trigger.building).unwrap();
    let translation = transform.translation;

    for maybe_slot in assignment.items.iter() {
        let slot_sprite = commands.spawn((
            Sprite {
                image: asset_server.load::<Image>("sprites/ui/slot.png"),
                ..Default::default()
            },
            translation.offset_x(50.).with_layer(Layers::UI),
        ));

        let Some(item) = maybe_slot else {
            continue;
        };

        // TODO: trait for sprite sheets?
        let sprite_sheet = match item.item_type {
            ItemType::Weapon(_) => weapons_sprite_sheet,
            ItemType::Chest => todo!(),
            ItemType::Feet => todo!(),
            ItemType::Head => todo!(),
        };

        let animation = sprite_sheet.animations.get(item.item_type);
        sprite.texture_atlas = Some(TextureAtlas {
            layout: sprite_sheet.layout.clone(),
            index: animation.first_sprite_index,
        });

        commands.spawn(
            (Sprite {
                image,
                ..Default::default()
            }),
        );

        slot_sprite.add_child(child);
    }
}
