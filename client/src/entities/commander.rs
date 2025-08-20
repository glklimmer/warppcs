use bevy::prelude::*;

use bevy::color::palettes::css::YELLOW;
use bevy_replicon::prelude::ClientTriggerExt;
use shared::{
    PlayerState, Vec3LayerExt,
    map::Layers,
    networking::UnitType,
    server::{
        buildings::recruiting::{Flag, FlagHolder},
        entities::commander::{
            ArmyFlagAssignments, ArmyFormation, CommanderAssignmentRequest,
            CommanderAssignmentResponse, CommanderCampInteraction, CommanderFormation,
            CommanderInteraction, CommanderPickFlag,
        },
    },
};

use crate::{
    animations::{
        objects::items::weapons::WeaponsSpriteSheet,
        ui::{
            animations::SpriteShaking,
            army_formations::{FormationIconSpriteSheet, FormationIcons},
            commander_menu::{CommanderMenuNodes, CommanderMenuSpriteSheet},
        },
    },
    networking::ControlledPlayer,
    widgets::menu::{
        CloseEvent, ClosedMenu, Menu, MenuNode, MenuPlugin, NodePayload, Selected, SelectionEvent,
    },
};

pub struct CommanderInteractionPlugin;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MainMenuEntries {
    Camp,
    Formation,
    Flag,
}

#[derive(Event, Deref)]
struct DrawHoverFlag(Entity);

#[derive(Component)]
struct HoverWeapon(UnitType);

#[derive(Component)]
struct HoverDisabledWeapon;

#[derive(Resource, Default, DerefMut, Deref)]
struct ActiveCommander(Option<Entity>);

impl Plugin for CommanderInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCommander>()
            .add_event::<DrawHoverFlag>()
            .add_observer(assignment_reject)
            .add_observer(open_commander_dialog)
            .add_observer(highligh_formation)
            .add_observer(open_create_camp)
            .add_observer(open_slots_dialog)
            .add_observer(send_selected)
            .add_observer(cleanup_menu_extras)
            .add_observer(draw_hovering_flag)
            .add_observer(assigment_warning)
            .add_observer(pick_commander_flag)
            .add_systems(
                Update,
                (formation_selected, update_flag_assignment)
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
    mut next_state: ResMut<NextState<PlayerState>>,
    mut active: ResMut<ActiveCommander>,
    menu_sprite_sheet: Res<CommanderMenuSpriteSheet>,
) {
    let commander = trigger.commander;

    let commander_position = transform.get(commander).unwrap();
    let texture = menu_sprite_sheet.sprite_sheet.texture.clone();

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
                MainMenuEntries::Flag,
                Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(
                        menu_sprite_sheet
                            .sprite_sheet
                            .texture_atlas(CommanderMenuNodes::Flag),
                    ),
                    custom_size: Some(Vec2::splat(15.)),
                    ..Default::default()
                },
            ),
            MenuNode::bundle(
                MainMenuEntries::Camp,
                Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(
                        menu_sprite_sheet
                            .sprite_sheet
                            .texture_atlas(CommanderMenuNodes::Camp),
                    ),
                    custom_size: Some(Vec2::splat(15.)),
                    ..Default::default()
                },
            ),
            MenuNode::bundle(
                MainMenuEntries::Formation,
                Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(
                        menu_sprite_sheet
                            .sprite_sheet
                            .texture_atlas(CommanderMenuNodes::Formation),
                    ),
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

fn pick_commander_flag(
    trigger: Trigger<SelectionEvent<MainMenuEntries>>,
    mut commands: Commands,
    mut close_menu: EventWriter<CloseEvent>,
) {
    let MainMenuEntries::Flag = trigger.selection else {
        return;
    };

    commands.client_trigger(CommanderPickFlag);
    close_menu.write(CloseEvent);
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
    let MainMenuEntries::Formation = trigger.selection else {
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

fn send_selected(trigger: Trigger<SelectionEvent<CommanderFormation>>, mut commands: Commands) {
    let SelectionEvent {
        selection: slot,
        menu: _,
        entry: _,
    } = *trigger;
    println!("Selected commander formation");

    commands.client_trigger(slot);
}

fn assignment_reject(
    trigger: Trigger<CommanderAssignmentResponse>,
    mut current_hover: Query<(Entity, &Transform), With<HoverWeapon>>,
    mut commands: Commands,
) {
    if CommanderAssignmentResponse::Reject != *trigger {
        return;
    }

    let Ok((hover_entity, transform)) = current_hover.single_mut() else {
        return;
    };

    let original_translation = transform.translation;
    commands
        .entity(hover_entity)
        .insert(SpriteShaking::new(0.1, 1.5, original_translation));
}

fn assigment_warning(
    trigger: Trigger<OnAdd, HoverWeapon>,
    unit_type: Query<&HoverWeapon>,
    hover_disabled_assignment: Query<Entity, With<HoverDisabledWeapon>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if unit_type
        .single()
        .is_ok_and(|flag| flag.0.eq(&UnitType::Commander))
    {
        match hover_disabled_assignment.single() {
            Ok(entity_to_despawn) => {
                commands.entity(entity_to_despawn).despawn();
            }
            Err(_) => {
                let disabled_flag_entity = commands
                    .spawn((
                        Sprite::from_image(asset_server.load("ui/commander/disabled.png")),
                        Transform::from_scale(Vec3::splat(1. / 3.)),
                    ))
                    .insert(HoverDisabledWeapon)
                    .id();

                commands
                    .entity(trigger.target())
                    .add_child(disabled_flag_entity);
            }
        }
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
        child.iter_descendants(active_commander).for_each(|entity| {
            commands
                .entity(entity)
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
                    MeshMaterial2d(materials.add(Color::from(YELLOW))),
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
    menu_entries_add: Query<&GlobalTransform>,
    mut current_hover: Query<&mut Transform, With<HoverWeapon>>,
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

    let Ok(entry_position) = menu_entries_add.get(trigger.0) else {
        return;
    };

    let player_flag = **flag_holder;
    let flag = flag.get(player_flag).unwrap();

    let weapon_sprite = weapons_sprite_sheet.sprite_for_unit(flag.unit_type);

    match current_hover.single_mut() {
        Ok(mut flag_position) => {
            flag_position.translation.x = entry_position.translation().x;
        }
        Err(_) => {
            let weapon_node_hover = commands.spawn(weapon_sprite).id();

            commands.entity(weapon_node_hover).insert((
                entry_position
                    .translation()
                    .offset_y(20.)
                    .with_layer(Layers::UI),
                HoverWeapon(flag.unit_type),
            ));
        }
    }
}

fn formation_selected(
    menu_entries_add: Query<Entity, (Added<Selected>, With<NodePayload<CommanderFormation>>)>,
    mut commands: Commands,
) {
    let Ok(entry) = menu_entries_add.single() else {
        return;
    };

    commands.client_trigger(CommanderAssignmentRequest);
    commands.trigger(DrawHoverFlag(entry));
}

fn update_flag_assignment(
    army_flag_assigments: Query<&ArmyFlagAssignments, Changed<ArmyFlagAssignments>>,
    menu_entries: Query<(Entity, &NodePayload<CommanderFormation>), With<Selected>>,
    current_hover: Query<Entity, With<HoverWeapon>>,
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

    if let Ok(current) = current_hover.single() {
        commands.entity(current).despawn();
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
