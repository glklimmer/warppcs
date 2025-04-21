use bevy::prelude::*;
use shared::{
    Vec3LayerExt,
    map::Layers,
    server::{
        buildings::item_assignment::{ItemAssignment, OpenItemAssignment, Slot},
        players::items::{ItemType, MeleeWeapon, ProjectileWeapon, WeaponType},
    },
};

use crate::animations::objects::items::{
    chests::ChestsSpriteSheet,
    feet::FeetSpriteSheet,
    heads::HeadsSpriteSheet,
    weapons::{Weapons, WeaponsSpriteSheet},
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

    for (i, (slot, maybe_item)) in assignment.items.iter_enums().enumerate() {
        let image = match slot {
            Slot::Weapon => "sprites/ui/slot_weapon.png",
            Slot::Chest => "sprites/ui/slot_chest.png",
            Slot::Feet => "sprites/ui/slot_feet.png",
            Slot::Head => "sprites/ui/slot_head.png",
        };
        let slot_sprite = commands
            .spawn((
                Sprite {
                    image: asset_server.load::<Image>(image),
                    ..Default::default()
                },
                translation
                    .offset_x(-37.5 + 25. * i as f32)
                    .offset_y(50.)
                    .with_layer(Layers::UI),
            ))
            .id();

        let Some(item) = maybe_item else {
            continue;
        };

        // TODO: trait for sprite sheets?
        // let sprite_sheet = match item.item_type {
        //     ItemType::Weapon(_) => weapons_sprite_sheet,
        //     ItemType::Chest => todo!(),
        //     ItemType::Feet => todo!(),
        //     ItemType::Head => todo!(),
        // };

        let ItemType::Weapon(weapon_type) = item.item_type else {
            continue;
        };
        let weapon_type = match weapon_type {
            WeaponType::Melee(use_weapon) => match use_weapon {
                MeleeWeapon::SwordAndShield => Weapons::SwordAndShield,
                MeleeWeapon::Pike => Weapons::Pike,
            },
            WeaponType::Projectile(projectile_weapon) => match projectile_weapon {
                ProjectileWeapon::Bow => Weapons::Bow,
            },
        };

        let sprite_sheet = &weapons_sprite_sheet.sprite_sheet;

        let animation = sprite_sheet.animations.get(weapon_type);

        let item_sprite = commands
            .spawn((
                Sprite {
                    image: sprite_sheet.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: sprite_sheet.layout.clone(),
                        index: animation.first_sprite_index,
                    }),
                    ..Default::default()
                },
                translation.with_layer(Layers::UI),
            ))
            .id();

        commands.entity(slot_sprite).add_child(item_sprite);
    }
}
