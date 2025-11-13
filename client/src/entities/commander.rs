use bevy::prelude::*;

use animations::{
    objects::items::weapons::WeaponsSpriteSheet,
    ui::{
        animations::SpriteShaking,
        army_formations::{FormationIconSpriteSheet, FormationIcons},
        commander_menu::{CommanderMenuNodes, CommanderMenuSpriteSheet},
    },
};
use bevy::color::palettes::css::YELLOW;
use bevy_replicon::prelude::ClientTriggerExt;
use shared::{
    ControlledPlayer, PlayerState, Vec3LayerExt,
    map::Layers,
    networking::UnitType,
    server::{
        buildings::recruiting::{Flag, FlagHolder},
        entities::commander::{
            ArmyFlagAssignments, ArmyFormation, ArmyPosition, CommanderAssignmentReject,
            CommanderAssignmentRequest, CommanderCampInteraction, CommanderInteraction,
            CommanderPickFlag,
        },
    },
};

use crate::widgets::menu::{
    CloseEvent, ClosedMenu, Menu, MenuNode, MenuPlugin, NodePayload, Selected, SelectionEvent,
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
            .add_observer(select_create_camp)
            .add_observer(open_slots_dialog)
            .add_observer(send_selected)
            .add_observer(cleanup_menu_extras)
            .add_observer(draw_hovering_flag)
            .add_observer(assigment_warning)
            .add_observer(select_commander_flag)
            .add_systems(
                Update,
                (formation_selected, update_flag_assignment)
                    .run_if(in_state(PlayerState::Interaction)),
            )
            .add_plugins((
                MenuPlugin::<MainMenuEntries>::default(),
                MenuPlugin::<ArmyPosition>::default(),
            ));
    }
}

fn open_commander_dialog(
    trigger: Trigger<CommanderInteraction>,
    transform: Query<&Transform>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut active: ResMut<ActiveCommander>,
    menu_sprite_sheet: Res<CommanderMenuSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let commander = trigger.commander;

    let commander_position = transform.get(commander)?;
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
    Ok(())
}

fn select_create_camp(trigger: Trigger<SelectionEvent<MainMenuEntries>>, mut commands: Commands) {
    let MainMenuEntries::Camp = trigger.selection else {
        return;
    };

    commands.client_trigger(CommanderCampInteraction);
}

fn select_commander_flag(
    trigger: Trigger<SelectionEvent<MainMenuEntries>>,
    mut close_menu: EventWriter<CloseEvent>,
    mut commands: Commands,
) {
    let MainMenuEntries::Flag = trigger.selection else {
        return;
    };

    commands.client_trigger(CommanderPickFlag);
    close_menu.write(CloseEvent);
}

fn open_slots_dialog(
    trigger: Trigger<SelectionEvent<MainMenuEntries>>,
    army_flag_assignments: Query<&ArmyFlagAssignments>,
    transform: Query<&GlobalTransform>,
    flag: Query<&Flag>,
    active: Res<ActiveCommander>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    formations: Res<FormationIconSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let MainMenuEntries::Formation = trigger.selection else {
        return Ok(());
    };

    let active_commander = (**active).ok_or("No active commander found")?;
    let entry_position = transform.get(trigger.entry)?.translation();
    let army_flag_assignments = army_flag_assignments.get(active_commander)?;
    let commander_facing = transform.get(active_commander)?.scale().x.signum();

    let menu_nodes: Vec<MenuNode<ArmyPosition>> = army_flag_assignments
        .flags
        .iter_enums()
        .map(|(formation, maybe_flag)| {
            let icon = match formation {
                ArmyPosition::Front => FormationIcons::Front,
                ArmyPosition::Middle => FormationIcons::Middle,
                ArmyPosition::Back => FormationIcons::Back,
            };
            let atlas = formations.sprite_sheet.texture_atlas(icon);
            let texture = formations.sprite_sheet.texture.clone();

            let maybe_weapon_sprite = maybe_flag.and_then(|flag_entity| {
                let Ok(flag) = flag.get(flag_entity) else {
                    return None;
                };
                let weapon_sprite = weapons_sprite_sheet.sprite_for_unit(flag.unit_type);
                let weapon_sprite_entity = commands
                    .spawn((
                        weapon_sprite,
                        Transform::from_xyz(0., 0., 1.).with_scale(Vec3::new(4., 4., 1.)),
                    ))
                    .id();
                Some(weapon_sprite_entity)
            });

            MenuNode::with_fn(formation, move |commands, entry| {
                let mut entry = commands.entity(entry);

                entry.insert(Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(atlas.clone()),
                    flip_x: commander_facing.is_sign_positive(),
                    ..Default::default()
                });

                if let Some(flag_weapon) = maybe_weapon_sprite {
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
    Ok(())
}

fn send_selected(trigger: Trigger<SelectionEvent<ArmyPosition>>, mut commands: Commands) {
    let SelectionEvent {
        selection: slot,
        menu: _,
        entry: _,
    } = *trigger;

    commands.client_trigger(slot);
}

fn assignment_reject(
    _: Trigger<CommanderAssignmentReject>,
    mut current_hover: Query<(Entity, &Transform), With<HoverWeapon>>,
    mut commands: Commands,
) -> Result {
    let (hover_entity, transform) = current_hover.single_mut()?;

    let original_translation = transform.translation;
    commands
        .entity(hover_entity)
        .insert(SpriteShaking::new(0.1, 1.5, original_translation));
    Ok(())
}

fn assigment_warning(
    trigger: Trigger<OnAdd, HoverWeapon>,
    unit_type: Query<&HoverWeapon>,
    hover_disabled_assignment: Query<Entity, With<HoverDisabledWeapon>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) -> Result {
    let unit = unit_type.single()?;

    if unit.0.eq(&UnitType::Commander) {
        match hover_disabled_assignment.single() {
            Ok(entity_to_despawn) => {
                commands.entity(entity_to_despawn).despawn();
            }
            Err(_) => {
                let disabled_sprite_entity = commands
                    .spawn((
                        Sprite::from_image(asset_server.load("ui/commander/disabled.png")),
                        Transform::from_scale(Vec3::splat(1. / 3.)),
                    ))
                    .insert(HoverDisabledWeapon)
                    .id();

                commands
                    .entity(trigger.target())
                    .add_child(disabled_sprite_entity);
            }
        }
    };
    Ok(())
}

fn cleanup_menu_extras(
    _: Trigger<ClosedMenu<ArmyPosition>>,
    current_hover: Query<Entity, With<HoverWeapon>>,
    mesh_highlights: Query<Entity, (With<Mesh2d>, With<MeshMaterial2d<ColorMaterial>>)>,
    mut commands: Commands,
) {
    for entity in mesh_highlights.iter() {
        commands
            .entity(entity)
            .remove::<(Mesh2d, MeshMaterial2d<ColorMaterial>)>();
    }

    if let Ok(current) = current_hover.single() {
        commands.entity(current).despawn();
    }
}

fn highligh_formation(
    trigger: Trigger<DrawHoverFlag>,
    menu_entries_add: Query<&NodePayload<ArmyPosition>>,
    army_formation: Query<&ArmyFormation>,
    active_commander: Res<ActiveCommander>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) -> Result {
    let selected_formation = menu_entries_add.get(**trigger)?;
    let active_commander = (**active_commander).ok_or("No active commander")?;
    let army_formations = army_formation.get(active_commander)?;

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
    Ok(())
}

fn draw_hovering_flag(
    trigger: Trigger<DrawHoverFlag>,
    menu_entries_add: Query<&GlobalTransform>,
    mut current_hover: Query<&mut Transform, With<HoverWeapon>>,
    player: Query<Option<&FlagHolder>, With<ControlledPlayer>>,
    flag: Query<&Flag>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    mut commands: Commands,
) -> Result {
    let maybe_flag_holder = player.single()?;
    let Some(flag_holder) = maybe_flag_holder else {
        return Ok(());
    };

    let entry_position = menu_entries_add.get(trigger.0)?;

    let player_flag = **flag_holder;
    let flag = flag.get(player_flag)?;

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
    Ok(())
}

fn formation_selected(
    menu_entries_add: Query<Entity, (Added<Selected>, With<NodePayload<ArmyPosition>>)>,
    mut commands: Commands,
) -> Result {
    let Ok(entry) = menu_entries_add.single() else {
        return Ok(());
    };

    commands.client_trigger(CommanderAssignmentRequest);
    commands.trigger(DrawHoverFlag(entry));
    Ok(())
}

fn update_flag_assignment(
    army_flag_assigments: Query<&ArmyFlagAssignments, Changed<ArmyFlagAssignments>>,
    menu_entries: Query<(Entity, &NodePayload<ArmyPosition>), With<Selected>>,
    current_hover: Query<Entity, With<HoverWeapon>>,
    active: Res<ActiveCommander>,
    weapons_sprite_sheet: Res<WeaponsSpriteSheet>,
    flag: Query<&Flag>,
    mut commands: Commands,
) -> Result {
    let Some(active_commander) = **active else {
        return Ok(());
    };

    let Ok(army_flag_assigment) = army_flag_assigments.get(active_commander) else {
        return Ok(());
    };

    let Ok((entry, selected_slot)) = menu_entries.single() else {
        return Ok(());
    };

    if let Ok(current) = current_hover.single() {
        commands.entity(current).despawn();
    };

    let maybe_flag_assigned = army_flag_assigment.flags.get(**selected_slot);

    let Some(flag_assigned) = maybe_flag_assigned else {
        commands.entity(entry).despawn_related::<Children>();
        commands.trigger(DrawHoverFlag(entry));
        return Ok(());
    };

    let flag = flag.get(*flag_assigned)?;
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
    Ok(())
}
