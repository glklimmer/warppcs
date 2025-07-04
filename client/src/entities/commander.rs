use bevy::prelude::*;

use bevy::color::palettes::css::PURPLE;
use bevy_replicon::prelude::ClientTriggerExt;
use shared::{
    PlayerState, Vec3LayerExt,
    map::Layers,
    server::{
        buildings::recruiting::{Flag, FlagHolder},
        entities::commander::{
            ArmyFlagAssignments, ArmyFormation, CommanderCampInteraction, CommanderFormation,
            CommanderInteraction,
        },
    },
};

use crate::{
    animations::{
        objects::items::weapons::WeaponsSpriteSheet,
        ui::army_formations::{FormationIconSpriteSheet, FormationIcons},
    },
    networking::ControlledPlayer,
    widgets::menu::{
        ClosedMenu, Menu, MenuNode, MenuPlugin, NodePayload, Selected, SelectionEvent,
    },
};

pub struct CommanderInteractionPlugin;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MainMenuEntries {
    Camp,
    Slots,
}

#[derive(Event, Deref)]
struct DrawHoverFlag(Entity);

#[derive(Component)]
struct HoverWeapon;

#[derive(Resource, Default, DerefMut, Deref)]
struct ActiveCommander(Option<Entity>);

impl Plugin for CommanderInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCommander>()
            .add_event::<DrawHoverFlag>()
            .add_observer(open_commander_dialog)
            .add_observer(highligh_formation)
            .add_observer(open_create_camp)
            .add_observer(open_slots_dialog)
            .add_observer(send_selected)
            .add_observer(cleanup_menu_extras)
            .add_observer(draw_hovering_flag)
            .add_systems(
                Update,
                (update_flag_assignment, draw_flag_on_selected)
                    .run_if(in_state(PlayerState::Interaction)),
            )
            .add_plugins((
                MenuPlugin::<MainMenuEntries>::default(),
                MenuPlugin::<CommanderFormation>::default(),
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
                MainMenuEntries::Camp,
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

fn open_create_camp(trigger: Trigger<SelectionEvent<MainMenuEntries>>, mut commands: Commands) {
    let MainMenuEntries::Camp = trigger.selection else {
        return;
    };

    commands.client_trigger(CommanderCampInteraction);
}

fn open_slots_dialog(
    trigger: Trigger<SelectionEvent<MainMenuEntries>>,
    mut commands: Commands,
    army_flag_assignments: Query<&ArmyFlagAssignments>,
    transform: Query<&GlobalTransform>,
    flag: Query<&Flag>,
    active: Res<ActiveCommander>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    formations: Res<FormationIconSpriteSheet>,
) {
    let MainMenuEntries::Slots = trigger.selection else {
        return;
    };

    let entry_position = transform.get(trigger.entry).unwrap().translation();
    let army_flag_assignments = army_flag_assignments.get(active.unwrap()).unwrap();
    let commander_facing = transform.get(active.unwrap()).unwrap().scale().x.signum();

    let menu_nodes: Vec<MenuNode<CommanderFormation>> = army_flag_assignments
        .flags
        .iter_enums()
        .map(|(formation, maybe_flag)| {
            let icon = match formation {
                CommanderFormation::Front => FormationIcons::Front,
                CommanderFormation::Middle => FormationIcons::Middle,
                CommanderFormation::Back => FormationIcons::Back,
            };
            let atlas = formations.sprite_sheet.texture_atlas(icon);
            let texture = formations.sprite_sheet.texture.clone();

            let has_unit_weapon = maybe_flag.map(|flag_entity| {
                let flag = flag.get(flag_entity).unwrap();
                let weapon_sprite = weapons_sprite_sheet.sprite_for_unit(flag.unit_type);
                commands
                    .spawn((
                        weapon_sprite,
                        Transform::from_xyz(0., 0., 1.).with_scale(Vec3::new(4., 4., 1.)),
                    ))
                    .id()
            });

            MenuNode::with_fn(formation, move |commands, entry| {
                let mut entry = commands.entity(entry);

                entry.insert(Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(atlas.clone()),
                    flip_x: commander_facing.is_sign_positive(),
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
        Menu::new(menu_nodes)
            .with_gap(15.)
            .with_entry_scale(1. / 5.),
    ));
}

fn send_selected(
    trigger: Trigger<SelectionEvent<CommanderFormation>>,
    current_hover: Query<Entity, With<HoverWeapon>>,
    mut commands: Commands,
) {
    let SelectionEvent {
        selection: slot,
        menu: _,
        entry: _,
    } = *trigger;

    commands.client_trigger(slot);
    if let Ok(current) = current_hover.single() {
        commands.entity(current).despawn();
    };
}

fn cleanup_menu_extras(
    _: Trigger<ClosedMenu<CommanderFormation>>,
    current_hover: Query<Entity, With<HoverWeapon>>,
    child: Query<&Children>,
    active: Res<ActiveCommander>,
    mut commands: Commands,
) {
    if let Some(active_commander) = **active {
        child.iter_descendants(active_commander).for_each(|each| {
            commands
                .entity(each)
                .remove::<(Mesh2d, MeshMaterial2d<ColorMaterial>)>();
        });
    }

    let Ok(current) = current_hover.single() else {
        return;
    };
    commands.entity(current).despawn();
}

fn highligh_formation(
    trigger: Trigger<DrawHoverFlag>,
    menu_entries_add: Query<&NodePayload<CommanderFormation>>,
    army_formation: Query<&ArmyFormation>,
    active: Res<ActiveCommander>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    let Ok(selected_formation) = menu_entries_add.get(**trigger) else {
        return;
    };

    let Ok(army_formations) = army_formation.get(active.unwrap()) else {
        return;
    };

    army_formations
        .positions
        .iter_enums()
        .for_each(|(formation, entity)| {
            if formation == **selected_formation {
                commands.entity(*entity).insert((
                    Mesh2d(meshes.add(Rectangle::default())),
                    MeshMaterial2d(materials.add(Color::from(PURPLE))),
                ));
            } else {
                commands
                    .entity(*entity)
                    .remove::<(Mesh2d, MeshMaterial2d<ColorMaterial>)>();
            }
        });
}

fn draw_hovering_flag(
    trigger: Trigger<DrawHoverFlag>,
    mut current_hover: Query<&mut Transform, With<HoverWeapon>>,
    menu_entries_add: Query<&GlobalTransform, With<NodePayload<CommanderFormation>>>,
    player: Query<Option<&FlagHolder>, With<ControlledPlayer>>,
    flag: Query<&Flag>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    mut commands: Commands,
) {
    let Ok(maybe_flag_holder) = player.single() else {
        return;
    };
    let Some(flag_holder) = maybe_flag_holder else {
        return;
    };
    let player_flag = **flag_holder;
    let flag = flag.get(player_flag).unwrap();
    let weapon_sprite = weapons_sprite_sheet.sprite_for_unit(flag.unit_type);

    let Ok(entry_position) = menu_entries_add.get(**trigger) else {
        return;
    };

    match current_hover.single_mut() {
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
    menu_entries_add: Query<Entity, (Added<Selected>, With<NodePayload<CommanderFormation>>)>,
    mut commands: Commands,
) {
    let Ok(entry) = menu_entries_add.single() else {
        return;
    };

    commands.trigger(DrawHoverFlag(entry));
}

fn update_flag_assignment(
    army_flag_assigments: Query<&ArmyFlagAssignments, Changed<ArmyFlagAssignments>>,
    menu_entries: Query<(Entity, &NodePayload<CommanderFormation>), With<Selected>>,
    active: Res<ActiveCommander>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    flag: Query<&Flag>,
    player: Query<&FlagHolder, With<ControlledPlayer>>,
    mut commands: Commands,
) {
    let Some(active_commander) = **active else {
        return;
    };

    let Ok(army_flag_assigment) = army_flag_assigments.get(active_commander) else {
        return;
    };

    let Ok((entry, selected_slot)) = menu_entries.single() else {
        return;
    };

    let maybe_flag_assigned = army_flag_assigment.flags.get(**selected_slot);
    let Some(flag_assigned) = maybe_flag_assigned else {
        let flag_holder = player.single().unwrap();
        let player_flag = **flag_holder;
        commands.entity(player_flag).insert(Visibility::Visible);
        commands.entity(entry).despawn_related::<Children>();
        commands.trigger(DrawHoverFlag(entry));
        return;
    };

    commands.entity(*flag_assigned).insert(Visibility::Hidden);

    let flag = flag.get(*flag_assigned).unwrap();
    let weapon_sprite = weapons_sprite_sheet.sprite_for_unit(flag.unit_type);

    let flag_weapon_slot = commands
        .spawn((
            weapon_sprite,
            Transform::from_xyz(0., 0., 1.).with_scale(Vec3::new(4., 4., 1.)),
        ))
        .id();

    commands
        .entity(entry)
        .despawn_related::<Children>()
        .add_child(flag_weapon_slot);

    // Flag maybe be swapped between player and unit.
    commands.trigger(DrawHoverFlag(entry));
}
