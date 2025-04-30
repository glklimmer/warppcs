use bevy::prelude::*;
use bevy_replicon::prelude::ClientTriggerExt;
use shared::{
    PlayerState, Vec3LayerExt,
    map::Layers,
    networking::UnitType,
    server::{
        buildings::recruiting::FlagAssignment,
        entities::{
            Unit,
            commander::{CommanderInteraction, SlotSelection, SlotsAssignments},
        },
    },
};

use crate::{
    animations::objects::items::weapons::{Weapons, WeaponsSpriteSheet},
    widgets::menu::{Menu, MenuNode, MenuPlugin, NodePayload, Selected, SelectionEvent},
};

use super::items::BuildSprite;

pub struct CommanderInteractionPlugin;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MainMenuEntries {
    Map,
    Slots,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    Front,
    Middle,
    Back,
}

#[derive(Resource, Default, DerefMut, Deref)]
struct ActiveCommander(Option<Entity>);

impl Plugin for CommanderInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCommander>();

        app.add_observer(open_commander_dialog)
            .add_observer(open_slots_dialog)
            .add_observer(slot_selected)
            .add_observer(send_selected)
            .add_plugins((
                MenuPlugin::<MainMenuEntries>::default(),
                MenuPlugin::<Slot>::default(),
            ));
    }
}

fn open_commander_dialog(
    trigger: Trigger<CommanderInteraction>,
    mut commands: Commands,
    transform: Query<(&SlotsAssignments, &Transform)>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut active: ResMut<ActiveCommander>,
) {
    let commander = trigger.commander;

    let map: Handle<Image> = asset_server.load("ui/commander/map.png");
    let slots: Handle<Image> = asset_server.load("ui/commander/slots.png");

    let (slot_assignments, unit_position) = transform.get(commander).unwrap();

    next_state.set(PlayerState::Interaction);

    commands.spawn((
        Visibility::default(),
        unit_position
            .translation
            .offset_x(-5.5)
            .offset_y(25.)
            .with_layer(Layers::Item),
        Menu::new(vec![
            MenuNode::bundle(MainMenuEntries::Map, Sprite::from_image(map)),
            MenuNode::bundle(MainMenuEntries::Slots, Sprite::from_image(slots)),
        ]),
    ));

    **active = Some(commander);
}

fn open_slots_dialog(
    trigger: Trigger<SelectionEvent<MainMenuEntries>>,
    mut commands: Commands,
    transform: Query<&GlobalTransform>,
    asset_server: Res<AssetServer>,
) {
    let MainMenuEntries::Slots = trigger.selection else {
        return;
    };
    let empty_slot: Handle<Image> = asset_server.load("ui/commander/slot_empty.png");

    let transform = transform.get(trigger.entry).unwrap().translation();

    commands.spawn((
        Visibility::default(),
        transform
            .offset_x(-5.5)
            .offset_y(25.)
            .with_layer(Layers::Item),
        Menu::new(vec![
            MenuNode::bundle(Slot::Front, Sprite::from_image(empty_slot.clone())),
            MenuNode::bundle(Slot::Middle, Sprite::from_image(empty_slot.clone())),
            MenuNode::bundle(Slot::Middle, Sprite::from_image(empty_slot)),
        ]),
    ));
}

fn send_selected(trigger: Trigger<SelectionEvent<Slot>>, mut commands: Commands) {
    let SelectionEvent {
        selection: slot,
        menu: _,
        entry,
    } = *trigger;

    match slot {
        Slot::Front => commands.client_trigger(SlotSelection::Front),
        Slot::Middle => commands.client_trigger(SlotSelection::Middle),
        Slot::Back => commands.client_trigger(SlotSelection::Back),
    };
}

fn slot_selected(
    slot_assigments: Query<&SlotsAssignments, Changed<SlotsAssignments>>,
    menu_entries: Query<(Entity, &NodePayload<Slot>), With<Selected>>,
    units_on_flag: Query<(&FlagAssignment, &Unit)>,
    active: Res<ActiveCommander>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
) {
    let Some(active_commander) = **active else {
        return;
    };

    let slots_assigment = slot_assigments.get(active_commander).unwrap();

    let flag_selcted = match slot {
        Slot::Front => slots_assigment.front,
        Slot::Middle => slots_assigment.front,
        Slot::Back => slots_assigment.front,
    };

    let unit = units_on_flag
        .iter()
        .find(|(assignment, _)| assignment.0 == flag_selcted.unwrap());

    let Some((_, unit)) = unit else { return };

    let weapon_sprite = match unit.unit_type {
        UnitType::Shieldwarrior => weapons_sprite_sheet
            .sprite_sheet
            .sprite_for(Weapons::SwordAndShield),
        UnitType::Pikeman => weapons_sprite_sheet.sprite_sheet.sprite_for(Weapons::Pike),
        UnitType::Archer => weapons_sprite_sheet.sprite_sheet.sprite_for(Weapons::Bow),
        UnitType::Bandit => todo!(),
        UnitType::Commander => todo!(),
    };
}
