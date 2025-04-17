use bevy::{
    color::palettes::css::GREY,
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use bevy_replicon::{client::ClientSet, prelude::ClientTriggerExt};
use shared::{
    Vec3LayerExt,
    map::Layers,
    player_movement::PlayerState,
    server::entities::commander::{CommanderInteraction, SlotSelection, SlotsAssignments},
};

pub struct QuickMenuPlugin;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MainCommands {
    Map,
    Slots,
    Test,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum AssignSlot {
    Front,
    Middle,
    Back,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MenuCommands {
    Main(MainCommands),
    Slot(AssignSlot),
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Next(MenuCommands);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Branches(MenuCommands);

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
        app.insert_state(PlayerState::World);
        app.add_observer(draw_options);
        app.add_systems(
            Update,
            (
                cycle_commands,
                branch_into_command,
                return_to_main_commands,
                send_selected.before(ClientSet::Send),
            )
                .run_if(in_state(PlayerState::Commands)),
        );
    }
}

fn draw_options(
    trigger: Trigger<CommanderInteraction>,
    mut commands: Commands,
    transform: Query<(&SlotsAssignments, &Transform)>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    let CommanderInteraction::Options(unit) = trigger.event() else {
        return;
    };

    let empty_slot: Handle<Image> = asset_server.load("ui/commander/slot_empty.png");
    let full_slot: Handle<Image> = asset_server.load("ui/commander/slot_full.png");

    let (slot_assignments, unit_position) = transform.get(*unit).unwrap();

    next_state.set(PlayerState::Commands);

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
        MainCommands::Map,
        Next(MenuCommands::Main(MainCommands::Slots)),
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
        MainCommands::Slots,
        Next(MenuCommands::Main(MainCommands::Test)),
        Branches(MenuCommands::Slot(AssignSlot::Front)),
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("ui/commander/map.png"),
            custom_size: Some(Vec2::new(10., 10.)),
            color: Color::Srgba(GREY),
            ..default()
        },
        unit_position
            .translation
            .offset_x(15.5)
            .offset_y(25.)
            .with_layer(Layers::Item),
        MainCommands::Test,
        Next(MenuCommands::Main(MainCommands::Map)),
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
            AssignSlot::Front,
            Next(MenuCommands::Slot(AssignSlot::Middle)),
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
            AssignSlot::Middle,
            Next(MenuCommands::Slot(AssignSlot::Back)),
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
            AssignSlot::Back,
            Next(MenuCommands::Slot(AssignSlot::Front)),
            Visibility::Hidden,
        ))
        .insert_if(Active, || slot_assignments.back.is_some());
}

fn cycle_commands(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    main: Query<(Entity, &MainCommands, &Next)>,
    slots: Query<(Entity, &AssignSlot, &Next)>,
    selected_query: Query<(Entity, &Next), With<Selected>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        let (entity, next) = selected_query.single();
        commands.entity(entity).remove::<Selected>();

        let nexton = match next.0 {
            MenuCommands::Main(main_options) => {
                main.iter()
                    .find(|(_, m, _)| m.eq(&&main_options))
                    .unwrap()
                    .0
            }
            MenuCommands::Slot(slot_options) => {
                slots
                    .iter()
                    .find(|(_, m, _)| m.eq(&&slot_options))
                    .unwrap()
                    .0
            }
        };
        commands.entity(nexton).insert(Selected);
    }
}

fn branch_into_command(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    slots: Query<(Entity, &AssignSlot, &Next)>,
    selected_query: Query<(Entity, Option<&Branches>), With<Selected>>,
) {
    if keyboard.just_pressed(KeyCode::KeyW) {
        let (entity, next) = selected_query.single();

        if next.is_none() {
            return;
        }

        let MenuCommands::Slot(variant) = next.unwrap().0 else {
            return;
        };

        commands.entity(entity).remove::<Selected>();

        slots.iter().for_each(|(e, s, _)| {
            commands
                .entity(e)
                .insert(Visibility::Visible)
                .insert_if(Selected, || s.eq(&variant));
        });
    }
}

fn return_to_main_commands(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    main_commands: Query<(Entity, &MainCommands, &Branches)>,
    assign_slots: Query<(Entity, &AssignSlot, &Next)>,
    selected_query: Query<(Entity, Option<&MainCommands>), With<Selected>>,
    all: Query<Entity, With<Next>>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        let (entity, main_command) = selected_query.single();

        if main_command.is_some() {
            for entity in all.iter() {
                commands.entity(entity).despawn_recursive();
                next_state.set(PlayerState::World);
            }
        } else {
            commands.entity(entity).remove::<Selected>();

            assign_slots.iter().for_each(|(e, _, _)| {
                commands.entity(e).insert(Visibility::Hidden);
            });

            main_commands
                .iter()
                .find(|(_, _, b)| b.0.eq(&MenuCommands::Slot(AssignSlot::Front)))
                .map(|(e, _, _)| {
                    commands.entity(e).insert(Selected);
                });
        }
    }
}

fn send_selected(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selected_query: Query<(&mut Sprite, Option<&AssignSlot>, Option<&Active>), With<Selected>>,
    asset_server: Res<AssetServer>,
) {
    if keyboard.just_pressed(KeyCode::KeyF) {
        let (mut sprite, slot, active) = selected_query.single_mut();

        let empty_slot: Handle<Image> = asset_server.load("ui/commander/slot_empty.png");
        let full_slot: Handle<Image> = asset_server.load("ui/commander/slot_full.png");

        match slot.unwrap() {
            AssignSlot::Front => commands.client_trigger(SlotSelection::Front),
            AssignSlot::Middle => commands.client_trigger(SlotSelection::Middle),
            AssignSlot::Back => commands.client_trigger(SlotSelection::Back),
        };

        let image = match active {
            Some(_) => empty_slot,
            None => full_slot,
        };

        sprite.image = image;
    }
}
