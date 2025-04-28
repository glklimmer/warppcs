use bevy::{
    color::palettes::css::GREY,
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use bevy_replicon::{client::ClientSet, prelude::ClientTriggerExt};
use shared::{
    PlayerState, Vec3LayerExt,
    map::Layers,
    server::entities::commander::{CommanderInteraction, SlotSelection, SlotsAssignments},
};

pub struct QuickMenuPlugin;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MainMenuEntries {
    Map,
    Slots,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Front,
    Middle,
    Back,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MenuEntries {
    Main(MainMenuEntries),
    Slot(Slot),
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct LeftEntry(MenuEntries);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct RightEntry(MenuEntries);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Branches(MenuEntries);

#[derive(Component, Clone, Copy)]
struct Active;

#[derive(Component)]
#[component(on_add = on_add_seleted)]
#[component(on_remove = on_remove_seleted)]
struct Selected;

fn on_add_seleted(mut world: DeferredWorld, entity: Entity, _id: ComponentId) {
    let mut entity_mut = world.entity_mut(entity);
    let mut sprite = entity_mut.get_mut::<Sprite>().unwrap();
    sprite.color = Color::WHITE;
}

fn on_remove_seleted(mut world: DeferredWorld, entity: Entity, _id: ComponentId) {
    let mut entity_mut = world.entity_mut(entity);
    let mut sprite = entity_mut.get_mut::<Sprite>().unwrap();
    sprite.color = Color::Srgba(GREY);
}

impl Plugin for QuickMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(draw_commander_interaction);
        app.add_systems(
            Update,
            (
                cycle_right_commands,
                cycle_left_commands,
                branch_into_command,
                return_to_main_menu,
                send_selected.before(ClientSet::Send),
            )
                .run_if(in_state(PlayerState::Interaction)),
        );
    }
}

fn draw_commander_interaction(
    trigger: Trigger<CommanderInteraction>,
    mut commands: Commands,
    transform: Query<(&SlotsAssignments, &Transform)>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    let commander = trigger.commander;

    let empty_slot: Handle<Image> = asset_server.load("ui/commander/slot_empty.png");
    let full_slot: Handle<Image> = asset_server.load("ui/commander/slot_full.png");

    let (slot_assignments, unit_position) = transform.get(commander).unwrap();

    next_state.set(PlayerState::Interaction);

    commands.spawn((
        Sprite {
            image: asset_server.load("ui/commander/map.png"),
            custom_size: Some(Vec2::new(10., 10.)),
            color: Color::Srgba(GREY),
            ..default()
        },
        unit_position
            .translation
            .offset_x(-5.5)
            .offset_y(25.)
            .with_layer(Layers::Item),
        MainMenuEntries::Map,
        LeftEntry(MenuEntries::Main(MainMenuEntries::Slots)),
        RightEntry(MenuEntries::Main(MainMenuEntries::Slots)),
        Branches(MenuEntries::Main(MainMenuEntries::Map)),
        Selected,
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("ui/commander/slots.png"),
            custom_size: Some(Vec2::new(10., 10.)),
            color: Color::Srgba(GREY),
            ..default()
        },
        unit_position
            .translation
            .offset_x(5.5)
            .offset_y(25.)
            .with_layer(Layers::Item),
        MainMenuEntries::Slots,
        LeftEntry(MenuEntries::Main(MainMenuEntries::Map)),
        RightEntry(MenuEntries::Main(MainMenuEntries::Map)),
        Branches(MenuEntries::Slot(Slot::Front)),
    ));

    let front = match slot_assignments.front {
        Some(_) => full_slot.clone(),
        None => empty_slot.clone(),
    };
    commands
        .spawn((
            Sprite {
                image: front,
                custom_size: Some(Vec2::new(10., 10.)),
                color: Color::Srgba(GREY),
                ..default()
            },
            unit_position
                .translation
                .offset_x(15.5)
                .offset_y(35.)
                .with_layer(Layers::Item),
            Slot::Front,
            LeftEntry(MenuEntries::Slot(Slot::Back)),
            RightEntry(MenuEntries::Slot(Slot::Middle)),
            Visibility::Hidden,
        ))
        .insert_if(Active, || slot_assignments.front.is_some());

    let middle = match slot_assignments.middle {
        Some(_) => full_slot.clone(),
        None => empty_slot.clone(),
    };
    commands
        .spawn((
            Sprite {
                image: middle.clone(),
                custom_size: Some(Vec2::new(10., 10.)),
                color: Color::Srgba(GREY),
                ..default()
            },
            unit_position
                .translation
                .offset_x(25.5)
                .offset_y(35.)
                .with_layer(Layers::Item),
            Slot::Middle,
            LeftEntry(MenuEntries::Slot(Slot::Front)),
            RightEntry(MenuEntries::Slot(Slot::Back)),
            Visibility::Hidden,
        ))
        .insert_if(Active, || slot_assignments.middle.is_some());

    let back = match slot_assignments.back {
        Some(_) => full_slot.clone(),
        None => empty_slot.clone(),
    };
    commands
        .spawn((
            Sprite {
                image: back,
                custom_size: Some(Vec2::new(10., 10.)),
                color: Color::Srgba(GREY),
                ..default()
            },
            unit_position
                .translation
                .offset_x(40.5)
                .offset_y(35.)
                .with_layer(Layers::Item),
            Slot::Back,
            LeftEntry(MenuEntries::Slot(Slot::Middle)),
            RightEntry(MenuEntries::Slot(Slot::Front)),
            Visibility::Hidden,
        ))
        .insert_if(Active, || slot_assignments.back.is_some());
}

fn cycle_right_commands(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    main: Query<(Entity, &MainMenuEntries, &RightEntry)>,
    slots: Query<(Entity, &Slot, &RightEntry)>,
    selected_query: Query<(Entity, &RightEntry), With<Selected>>,
) {
    if keyboard.just_pressed(KeyCode::KeyD) {
        let (entity, right) = selected_query.single();
        commands.entity(entity).remove::<Selected>();

        let right_menu_entry = match right.0 {
            MenuEntries::Main(main_options) => {
                main.iter()
                    .find(|(_, m, _)| m.eq(&&main_options))
                    .unwrap()
                    .0
            }
            MenuEntries::Slot(slot_options) => {
                slots
                    .iter()
                    .find(|(_, m, _)| m.eq(&&slot_options))
                    .unwrap()
                    .0
            }
        };
        commands.entity(right_menu_entry).insert(Selected);
    }
}

fn cycle_left_commands(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    main: Query<(Entity, &MainMenuEntries, &LeftEntry)>,
    slots: Query<(Entity, &Slot, &LeftEntry)>,
    selected_query: Query<(Entity, &LeftEntry), With<Selected>>,
) {
    if keyboard.just_pressed(KeyCode::KeyA) {
        let (entity, left) = selected_query.single();
        commands.entity(entity).remove::<Selected>();

        let left_menu_entry = match left.0 {
            MenuEntries::Main(main_options) => {
                main.iter()
                    .find(|(_, m, _)| m.eq(&&main_options))
                    .unwrap()
                    .0
            }
            MenuEntries::Slot(slot_options) => {
                slots
                    .iter()
                    .find(|(_, m, _)| m.eq(&&slot_options))
                    .unwrap()
                    .0
            }
        };
        commands.entity(left_menu_entry).insert(Selected);
    }
}

fn branch_into_command(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    slots: Query<(Entity, &Slot)>,
    selected_query: Query<(Entity, Option<&Branches>), With<Selected>>,
) {
    if keyboard.just_pressed(KeyCode::KeyW) {
        let (entity, branch) = selected_query.single();

        if branch.is_none() {
            return;
        }

        let MenuEntries::Slot(variant) = branch.unwrap().0 else {
            return;
        };

        commands.entity(entity).remove::<Selected>();

        slots.iter().for_each(|(e, s)| {
            commands
                .entity(e)
                .insert(Visibility::Visible)
                .insert_if(Selected, || s.eq(&variant));
        });
    }
}

fn return_to_main_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    main_menu_entries: Query<(Entity, &MainMenuEntries, &Branches)>,
    slots: Query<(Entity, &Slot, &RightEntry)>,
    selected_query: Query<(Entity, Option<&MainMenuEntries>), With<Selected>>,
    all: Query<Entity, With<RightEntry>>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        let (entity, main_menu) = selected_query.single();

        if main_menu.is_some() {
            for entity in all.iter() {
                commands.entity(entity).despawn_recursive();
                next_state.set(PlayerState::World);
            }
        } else {
            commands.entity(entity).remove::<Selected>();

            slots.iter().for_each(|(e, _, _)| {
                commands.entity(e).insert(Visibility::Hidden);
            });

            main_menu_entries
                .iter()
                .find(|(_, _, branch)| {
                    if let MenuEntries::Slot(_) = branch.0 {
                        true
                    } else {
                        false
                    }
                })
                .map(|(e, _, _)| {
                    commands.entity(e).insert(Selected);
                });
        }
    }
}

fn send_selected(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selected_query: Query<(&mut Sprite, Option<&Slot>, Option<&Active>), With<Selected>>,
    asset_server: Res<AssetServer>,
) {
    if keyboard.just_pressed(KeyCode::KeyF) {
        let (mut sprite, slot, active) = selected_query.single_mut();

        if slot.is_none() {
            return;
        }

        let empty_slot: Handle<Image> = asset_server.load("ui/commander/slot_empty.png");
        let full_slot: Handle<Image> = asset_server.load("ui/commander/slot_full.png");

        match slot.unwrap() {
            Slot::Front => commands.client_trigger(SlotSelection::Front),
            Slot::Middle => commands.client_trigger(SlotSelection::Middle),
            Slot::Back => commands.client_trigger(SlotSelection::Back),
        };

        let image = match active {
            Some(_) => empty_slot,
            None => full_slot,
        };

        sprite.image = image;
    }
}
