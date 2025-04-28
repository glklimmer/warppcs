use bevy::{color::palettes::css::GREY, prelude::*};

use bevy_replicon::prelude::ClientTriggerExt;
use shared::{
    PlayerState, Vec3LayerExt,
    map::Layers,
    networking::Inventory,
    server::{
        buildings::item_assignment::{
            AssignItem, CloseBuildingDialog, ItemAssignment, ItemSlot, OpenBuildingDialog,
            StartBuild,
        },
        players::items::{Item, ItemType},
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
    entities::items::ItemSprite,
    networking::ControlledPlayer,
    widgets::menu::{
        CloseEvent, Menu, MenuNode, MenuPlugin, NodePayload, Selected, SelectionEvent,
    },
};

use super::items::BuildSprite;

pub struct ItemAssignmentPlugin;

impl Plugin for ItemAssignmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentBuilding>()
            .add_observer(open_building_dialog)
            .add_observer(open_assignment_dialog)
            .add_observer(close_assignment_dialog)
            .add_observer(slot_selected)
            .add_observer(item_selected)
            .add_observer(start_build)
            .add_plugins((
                MenuPlugin::<BuildingDialog>::default(),
                MenuPlugin::<ItemSlot>::default(),
                MenuPlugin::<Item>::default(),
            ))
            .add_systems(Update, update_assignment);
    }
}

#[derive(Clone)]
enum BuildingDialog {
    Build,
    ItemSlots,
}

#[derive(Resource, Default, Deref, DerefMut)]
struct CurrentBuilding(Option<Entity>);

fn item_selected(trigger: Trigger<SelectionEvent<Item>>, mut commands: Commands) {
    let item = &trigger.selection;
    println!("Item selected: {:?}", item);

    commands.client_trigger(AssignItem::new(item.clone()));
    commands.send_event(CloseEvent);
}

fn slot_selected(
    trigger: Trigger<SelectionEvent<ItemSlot>>,
    mut commands: Commands,
    inventory: Query<&Inventory, With<ControlledPlayer>>,
    transform: Query<&GlobalTransform>,
    weapons_ss: Res<WeaponsSpriteSheet>,
    chests_ss: Res<ChestsSpriteSheet>,
    heads_ss: Res<HeadsSpriteSheet>,
    feet_ss: Res<FeetSpriteSheet>,
) {
    let SelectionEvent {
        selection: slot,
        menu: _,
        entry,
    } = *trigger;
    let items = &inventory.single().items;
    let transform = transform.get(entry).unwrap().translation();

    let sheets = SpriteSheets {
        weapons: &weapons_ss.sprite_sheet,
        chests: &chests_ss.sprite_sheet,
        heads: &heads_ss.sprite_sheet,
        feet: &feet_ss.sprite_sheet,
    };

    let nodes: Vec<_> = items
        .iter()
        .filter(|item| item.slot() == slot)
        .map(|item| build_node(item, &sheets))
        .collect();

    commands.spawn((
        Visibility::default(),
        transform.offset_y(25.).with_layer(Layers::UI),
        Menu::new(nodes),
    ));
}

struct SpriteSheets<'a> {
    weapons: &'a SpriteSheet<Weapons>,
    chests: &'a SpriteSheet<Chests>,
    heads: &'a SpriteSheet<Heads>,
    feet: &'a SpriteSheet<Feet>,
}

fn build_node(item: &Item, sheets: &SpriteSheets) -> MenuNode<Item> {
    let sprite = match item.item_type {
        ItemType::Weapon(wt) => sheets.weapons.sprite_for(wt),
        ItemType::Chest => sheets.chests.sprite_for(item.color.unwrap()),
        ItemType::Head => sheets.heads.sprite_for(item.color.unwrap()),
        ItemType::Feet => sheets.feet.sprite_for(item.color.unwrap()),
    };

    MenuNode::bundle(item.clone(), sprite)
}

fn open_building_dialog(
    trigger: Trigger<OpenBuildingDialog>,
    mut commands: Commands,
    mut current_building: ResMut<CurrentBuilding>,
    building: Query<&Transform>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    let transform = building.get(trigger.building).unwrap();
    let translation = transform.translation;

    commands.spawn((
        Visibility::default(),
        translation.offset_y(50.).with_layer(Layers::UI),
        Menu::new(vec![
            MenuNode::bundle(
                BuildingDialog::Build,
                Sprite {
                    image: asset_server.load::<Image>("sprites/ui/build_entry.png"),
                    ..Default::default()
                },
            ),
            MenuNode::bundle(
                BuildingDialog::ItemSlots,
                Sprite {
                    image: asset_server.load::<Image>("sprites/ui/items_entry.png"),
                    ..Default::default()
                },
            ),
        ]),
    ));

    **current_building = Some(trigger.building);
    next_state.set(PlayerState::Interaction);
}

fn start_build(trigger: Trigger<SelectionEvent<BuildingDialog>>, mut commands: Commands) {
    let BuildingDialog::Build = trigger.selection else {
        return;
    };

    commands.client_trigger(StartBuild);
}

fn close_assignment_dialog(trigger: Trigger<CloseBuildingDialog>, mut commands: Commands) {
    trigger;
    commands.send_event(CloseEvent);
}

fn update_assignment(
    query: Query<&ItemAssignment, Changed<ItemAssignment>>,
    mut commands: Commands,
    menu_entries: Query<(Entity, &NodePayload<ItemSlot>, Option<&Selected>)>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    chests_sprite_sheet: Res<ChestsSpriteSheet>,
    heads_sprite_sheet: Res<HeadsSpriteSheet>,
    feet_sprite_sheet: Res<FeetSpriteSheet>,
) {
    let Some(item_assignment) = query.iter().next() else {
        return;
    };

    for maybe_item in item_assignment.items.iter() {
        let Some(item) = maybe_item else {
            continue;
        };

        let Some((entry, _, maybe_selected)) = menu_entries
            .iter()
            .find(|(_, slot, _)| ***slot == item.slot())
        else {
            continue;
        };
        let mut sprite = item.sprite(
            &weapons_sprite_sheet,
            &chests_sprite_sheet,
            &feet_sprite_sheet,
            &heads_sprite_sheet,
        );
        match maybe_selected {
            Some(_) => sprite.color = Color::WHITE,
            None => sprite.color = Color::Srgba(GREY),
        }

        let item_sprite = commands
            .spawn((sprite, Transform::from_xyz(0., 0., 1.)))
            .id();

        let mut entity = commands.entity(entry);
        entity.despawn_descendants().add_child(item_sprite);
    }
}

fn open_assignment_dialog(
    trigger: Trigger<SelectionEvent<BuildingDialog>>,
    mut commands: Commands,
    current_building: Res<CurrentBuilding>,
    assignment: Query<&ItemAssignment>,
    transform: Query<&GlobalTransform>,
    asset_server: Res<AssetServer>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    chests_sprite_sheet: Res<ChestsSpriteSheet>,
    heads_sprite_sheet: Res<HeadsSpriteSheet>,
    feet_sprite_sheet: Res<FeetSpriteSheet>,
) {
    let BuildingDialog::ItemSlots = trigger.selection else {
        return;
    };

    let assignment = assignment.get((**current_building).unwrap()).unwrap();
    let transform = transform.get(trigger.entry).unwrap();
    let translation = transform.translation();

    let nodes = assignment
        .items
        .iter_enums()
        .map(|(slot, maybe_item)| {
            let str = match slot {
                ItemSlot::Weapon => "sprites/ui/slot_weapon.png",
                ItemSlot::Chest => "sprites/ui/slot_chest.png",
                ItemSlot::Head => "sprites/ui/slot_head.png",
                ItemSlot::Feet => "sprites/ui/slot_feet.png",
            };
            let slot_image = asset_server.load::<Image>(str);

            let item_sprite = maybe_item.as_ref().map(|item| {
                item.sprite(
                    &weapons_sprite_sheet,
                    &chests_sprite_sheet,
                    &feet_sprite_sheet,
                    &heads_sprite_sheet,
                )
            });

            MenuNode::with_fn(slot, move |commands, entry| {
                let maybe_item_entity: Option<Entity> = item_sprite.as_ref().map(|sprite| {
                    commands
                        .spawn((sprite.clone(), Transform::from_xyz(0., 0., 1.)))
                        .id()
                });

                let mut entry = commands.entity(entry);
                entry.insert(Sprite {
                    image: slot_image.clone(),
                    ..Default::default()
                });

                if let Some(child) = maybe_item_entity {
                    entry.add_child(child);
                }
            })
        })
        .collect();

    commands.spawn((
        Visibility::default(),
        Menu::new(nodes),
        translation.offset_y(25.).with_layer(Layers::UI),
    ));
}
