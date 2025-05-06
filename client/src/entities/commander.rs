use bevy::prelude::*;
use bevy_replicon::prelude::ClientTriggerExt;
use shared::{
    PlayerState, Vec3LayerExt,
    map::Layers,
    networking::UnitType,
    server::{
        buildings::recruiting::{FlagAssignment, FlagHolder},
        entities::{
            Unit,
            commander::{CommanderInteraction, SlotSelection, SlotsAssignments},
        },
    },
};

use crate::{
    animations::objects::items::weapons::{Weapons, WeaponsSpriteSheet},
    networking::ControlledPlayer,
    widgets::menu::{
        ClosedMenu, Menu, MenuNode, MenuPlugin, NodePayload, Selected, SelectionEvent,
    },
};

use super::items::BuildSprite;

pub struct CommanderInteractionPlugin;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MainMenuEntries {
    Map,
    Slots,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Slot {
    Front,
    Middle,
    Back,
}

#[derive(Event)]
struct DrawHoverFlag(Entity);

#[derive(Component)]
struct HoverWeapon;

#[derive(Resource, Default, DerefMut, Deref)]
struct ActiveCommander(Option<Entity>);

impl Plugin for CommanderInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCommander>();
        app.add_event::<DrawHoverFlag>();
        app.add_observer(open_commander_dialog)
            .add_observer(open_slots_dialog)
            .add_observer(send_selected)
            .add_observer(despawn_hover_weapon)
            .add_observer(draw_flag)
            .add_systems(
                Update,
                (assign_unit, draw_flag_on_selected).run_if(in_state(PlayerState::Interaction)),
            )
            .add_plugins((
                MenuPlugin::<MainMenuEntries>::default(),
                MenuPlugin::<Slot>::default(),
            ));
    }
}

fn open_commander_dialog(
    trigger: Trigger<CommanderInteraction>,
    mut commands: Commands,
    transform: Query<&Transform>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut active: ResMut<ActiveCommander>,
) {
    let commander = trigger.commander;

    let map: Handle<Image> = asset_server.load("ui/commander/map.png");
    let slots: Handle<Image> = asset_server.load("ui/commander/slots.png");

    let commander_position = transform.get(commander).unwrap();

    next_state.set(PlayerState::Interaction);

    commands.spawn((
        Visibility::default(),
        commander_position
            .translation
            .offset_x(-5.5)
            .offset_y(25.)
            .with_layer(Layers::Item),
        Menu::new(vec![
            MenuNode::bundle(
                MainMenuEntries::Map,
                Sprite {
                    image: map,
                    custom_size: Some(Vec2::splat(15.)),
                    ..Default::default()
                },
            ),
            MenuNode::bundle(
                MainMenuEntries::Slots,
                Sprite {
                    image: slots,
                    custom_size: Some(Vec2::splat(15.)),
                    ..Default::default()
                },
            ),
        ])
        .with_gap(15.),
    ));

    **active = Some(commander);
}

fn open_slots_dialog(
    trigger: Trigger<SelectionEvent<MainMenuEntries>>,
    mut commands: Commands,
    commander: Query<&SlotsAssignments>,
    transform: Query<&GlobalTransform>,
    asset_server: Res<AssetServer>,
    units_on_flag: Query<(&FlagAssignment, &Unit)>,
    active: Res<ActiveCommander>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
) {
    let MainMenuEntries::Slots = trigger.selection else {
        return;
    };

    let entry_position = transform.get(trigger.entry).unwrap().translation();
    let slot_assignments = commander.get(active.unwrap()).unwrap();

    let menu_nodes = vec![
        (Slot::Front, slot_assignments.front),
        (Slot::Middle, slot_assignments.middle),
        (Slot::Back, slot_assignments.back),
    ]
    .into_iter()
    .map(|(slot, slot_assignment)| {
        let empty_slot = asset_server.load::<Image>("ui/commander/slot_empty.png");
        let mut has_unit_weapon = None;
        if let Some(slot) = slot_assignment {
            let unit = units_on_flag
                .iter()
                .find(|(assignment, _)| assignment.0 == slot)
                .unwrap()
                .1;

            let weapon_sprite = match unit.unit_type {
                UnitType::Shieldwarrior => weapons_sprite_sheet
                    .sprite_sheet
                    .sprite_for(Weapons::SwordAndShield),
                UnitType::Pikeman => weapons_sprite_sheet.sprite_sheet.sprite_for(Weapons::Pike),
                UnitType::Archer => weapons_sprite_sheet.sprite_sheet.sprite_for(Weapons::Bow),
                UnitType::Bandit => todo!(),
                UnitType::Commander => todo!(),
            };
            let flag_weapon = commands
                .spawn((weapon_sprite, Transform::from_xyz(0., 0., 1.)))
                .id();
            has_unit_weapon = Some(flag_weapon);
        };

        MenuNode::with_fn(slot, move |commands, entry| {
            let mut entry = commands.entity(entry);
            entry.insert(Sprite {
                image: empty_slot.clone(),
                custom_size: Some(Vec2::splat(10.)),
                ..Default::default()
            });

            if let Some(flag_weapon) = has_unit_weapon {
                entry.add_child(flag_weapon);
            }
        })
    })
    .collect();

    commands.spawn((
        Visibility::default(),
        entry_position
            .offset_x(-5.5)
            .offset_y(15.)
            .with_layer(Layers::Item),
        Menu::new(menu_nodes).with_gap(10.),
    ));
}

fn send_selected(
    trigger: Trigger<SelectionEvent<Slot>>,
    current_hover: Query<Entity, With<HoverWeapon>>,
    mut commands: Commands,
) {
    let SelectionEvent {
        selection: slot,
        menu: _,
        entry: _,
    } = *trigger;

    match slot {
        Slot::Front => commands.client_trigger(SlotSelection::Front),
        Slot::Middle => commands.client_trigger(SlotSelection::Middle),
        Slot::Back => commands.client_trigger(SlotSelection::Back),
    };
    if let Ok(current) = current_hover.get_single() {
        commands.entity(current).despawn_recursive();
    };
}

fn despawn_hover_weapon(
    trigger: Trigger<ClosedMenu<Slot>>,
    current_hover: Query<Entity, With<HoverWeapon>>,
    mut commands: Commands,
) {
    trigger;
    let Ok(current) = current_hover.get_single() else {
        return;
    };
    commands.entity(current).despawn_recursive();
}

fn draw_flag(
    trigger: Trigger<DrawHoverFlag>,
    mut current_hover: Query<&mut Transform, With<HoverWeapon>>,
    menu_entries_add: Query<&GlobalTransform, With<NodePayload<Slot>>>,
    units_on_flag: Query<(&FlagAssignment, &Unit)>,
    player_flag: Query<Option<&FlagHolder>, With<ControlledPlayer>>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    mut commands: Commands,
) {
    let Ok(maybe_player_flag) = player_flag.get_single() else {
        return;
    };

    let Some(player_flag) = maybe_player_flag else {
        return;
    };

    let Ok(entry_position) = menu_entries_add.get(trigger.0) else {
        return;
    };

    let unit = units_on_flag
        .iter()
        .find(|(assignment, _)| assignment.0 == player_flag.0)
        .unwrap()
        .1;

    let weapon_sprite = match unit.unit_type {
        UnitType::Shieldwarrior => weapons_sprite_sheet
            .sprite_sheet
            .sprite_for(Weapons::SwordAndShield),
        UnitType::Pikeman => weapons_sprite_sheet.sprite_sheet.sprite_for(Weapons::Pike),
        UnitType::Archer => weapons_sprite_sheet.sprite_sheet.sprite_for(Weapons::Bow),
        UnitType::Bandit => todo!(),
        UnitType::Commander => todo!(),
    };

    match current_hover.get_single_mut() {
        Ok(mut flag_position) => {
            flag_position.translation.x = entry_position.translation().x;
        }
        Err(_) => {
            commands.spawn((
                weapon_sprite,
                entry_position
                    .translation()
                    .offset_y(20.)
                    .with_layer(Layers::UI),
                HoverWeapon,
            ));
        }
    }
}

fn draw_flag_on_selected(
    menu_entries_add: Query<Entity, (Added<Selected>, With<NodePayload<Slot>>)>,
    mut commands: Commands,
) {
    let Ok(entry) = menu_entries_add.get_single() else {
        return;
    };

    commands.trigger(DrawHoverFlag(entry));
}

fn assign_unit(
    slot_assigments: Query<&SlotsAssignments, Changed<SlotsAssignments>>,
    menu_entries: Query<(Entity, &NodePayload<Slot>), With<Selected>>,
    units_on_flag: Query<(&FlagAssignment, &Unit)>,
    active: Res<ActiveCommander>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    mut commands: Commands,
) {
    let Some(active_commander) = **active else {
        return;
    };

    let Ok(slots_assigment) = slot_assigments.get(active_commander) else {
        return;
    };

    let Ok((entry, slot)) = menu_entries.get_single() else {
        return;
    };

    let maybe_flag_assigned = match **slot {
        Slot::Front => slots_assigment.front,
        Slot::Middle => slots_assigment.middle,
        Slot::Back => slots_assigment.back,
    };

    let Some(flag_assigned) = maybe_flag_assigned else {
        commands.entity(entry).despawn_descendants();
        commands.trigger(DrawHoverFlag(entry));
        return;
    };

    let unit = units_on_flag
        .iter()
        .find(|(assignment, _)| assignment.0 == flag_assigned)
        .unwrap()
        .1;

    let unit_weapon = match unit.unit_type {
        UnitType::Shieldwarrior => weapons_sprite_sheet
            .sprite_sheet
            .sprite_for(Weapons::SwordAndShield),
        UnitType::Pikeman => weapons_sprite_sheet.sprite_sheet.sprite_for(Weapons::Pike),
        UnitType::Archer => weapons_sprite_sheet.sprite_sheet.sprite_for(Weapons::Bow),
        UnitType::Bandit => todo!(),
        UnitType::Commander => todo!(),
    };

    let flag_weapon_slot = commands
        .spawn((unit_weapon, Transform::from_xyz(0., 0., 1.)))
        .id();

    commands
        .entity(entry)
        .despawn_descendants()
        .add_child(flag_weapon_slot);

    // Flag maybe be swapped between player and unit.
    commands.trigger(DrawHoverFlag(entry));
}
