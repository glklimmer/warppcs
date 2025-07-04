use bevy::ecs::component::HookContext;
use bevy::prelude::*;

use bevy::{
    color::palettes::css::GREY, ecs::world::DeferredWorld,
    input::common_conditions::input_just_pressed,
};
use shared::{PlayerState, Vec3LayerExt, map::Layers};
use std::{marker::PhantomData, sync::Arc};

pub struct MenuPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T: Clone + Send + Sync + 'static> Default for MenuPlugin<T> {
    fn default() -> Self {
        MenuPlugin {
            _marker: PhantomData,
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Plugin for MenuPlugin<T> {
    fn build(&self, app: &mut App) {
        if app.world().get_resource::<MenuSingleton>().is_none() {
            app.add_systems(
                Update,
                close_menu.run_if(in_state(PlayerState::Interaction)),
            );
        }

        app.init_resource::<ActiveMenus>()
            .init_resource::<MenuSingleton>()
            .add_observer(open_menu::<T>)
            .add_event::<CloseEvent>()
            .add_event::<ClosedMenu<T>>()
            .add_systems(
                Update,
                (
                    selection_callback::<T>.run_if(input_just_pressed(KeyCode::KeyF)),
                    cycle_commands,
                    close_menu_trigger::<T>,
                )
                    .run_if(in_state(PlayerState::Interaction)),
            );
    }
}

#[derive(Resource, Default)]
struct MenuSingleton;

#[derive(Component)]
#[component(on_add = color_tree_white)]
#[component(on_remove = color_tree_grey)]
pub struct Selected;

#[derive(Component)]
#[component(on_add = color_tree_grey)]
struct GrayOnSpawn;

fn color_tree_white(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    color_tree(&mut world, entity, Color::WHITE);
}

fn color_tree_grey(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    color_tree(&mut world, entity, Color::Srgba(GREY));
}

fn color_tree(world: &mut DeferredWorld, root: Entity, color: Color) {
    let mut stack = vec![root];

    while let Some(entity) = stack.pop() {
        if let Some(mut sprite) = world.entity_mut(entity).get_mut::<Sprite>() {
            sprite.color = color;
        }

        let Some(children) = world.entity(entity).get::<Children>() else {
            continue;
        };

        for child in children.iter() {
            stack.push(child);
        }
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct Menu<T: Clone + 'static> {
    nodes: Vec<MenuNode<T>>,
    config: MenuConfig,
}
impl<T: Clone + 'static> Menu<T> {
    pub(crate) fn new(nodes: Vec<MenuNode<T>>) -> Self {
        Self {
            nodes,
            config: MenuConfig::default(),
        }
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.config.gap = gap;
        self
    }
}

struct MenuConfig {
    gap: f32,
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self { gap: 25. }
    }
}

pub struct MenuNode<T> {
    payload: T,
    spawn_fn: Arc<dyn Fn(&mut Commands, Entity) + Send + Sync>,
}

impl<T> MenuNode<T> {
    pub fn bundle<B>(payload: T, bundle: B) -> Self
    where
        B: Bundle + Clone + Send + Sync + 'static,
    {
        let spawn_fn = Arc::new(move |commands: &mut Commands, entry: Entity| {
            commands.entity(entry).insert(bundle.clone());
        });
        MenuNode { payload, spawn_fn }
    }

    pub fn with_fn<F>(payload: T, f: F) -> Self
    where
        F: Fn(&mut Commands, Entity) + Send + Sync + 'static,
    {
        MenuNode {
            payload,
            spawn_fn: Arc::new(f),
        }
    }
}

#[derive(Component, Deref)]
pub struct NodePayload<T>(T);

fn open_menu<T: Clone + Send + Sync + 'static>(
    trigger: Trigger<OnAdd, Menu<T>>,
    mut commands: Commands,
    mut active: ResMut<ActiveMenus>,
    query: Query<&Menu<T>>,
) {
    let menu_entity = trigger.target();
    let menu = query.get(menu_entity).unwrap();

    let len = menu.nodes.len();
    let mut offset = -((len / 2) as f32 * menu.config.gap);
    if len % 2 == 0 {
        offset += menu.config.gap / 2.;
    }

    for (i, node) in menu.nodes.iter().enumerate() {
        let entry = commands
            .spawn((
                NodePayload(node.payload.clone()),
                Vec3::ZERO
                    .offset_x(offset + menu.config.gap * i as f32)
                    .with_layer(Layers::UI),
            ))
            .id();

        (node.spawn_fn)(&mut commands, entry);

        commands
            .entity(entry)
            .insert(GrayOnSpawn)
            .insert_if(Selected, || i == 0);

        commands.entity(menu_entity).add_child(entry);
    }

    active.push(menu_entity);
}

#[derive(Event)]
pub struct NodeSelected<T>(PhantomData<T>);

fn cycle_commands(
    input: Res<ButtonInput<KeyCode>>,
    active_menu: Res<ActiveMenus>,
    active: Query<Entity, With<Selected>>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    let direction = if input.any_just_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        1
    } else if input.any_just_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        -1
    } else {
        return;
    };

    let Some(active_menu) = active_menu.last() else {
        return;
    };
    let Ok(descendants) = children_query.get(*active_menu) else {
        return;
    };

    let total_items = descendants.len() as i32;
    if total_items == 0 {
        return;
    }

    let mut current_index: i32 = 0;
    for (i, menu_node) in descendants.iter().enumerate() {
        if active.contains(menu_node) {
            current_index = i as i32;
            commands.entity(menu_node).remove::<Selected>();
            break;
        }
    }

    let next_index = (current_index + direction).rem_euclid(total_items);

    if let Some(next_entity) = descendants.get(next_index as usize) {
        commands.entity(*next_entity).insert(Selected);
    }
}

#[derive(Event)]
pub struct CloseEvent;

fn close_menu(
    mut close_events: EventReader<CloseEvent>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut active_menus: ResMut<ActiveMenus>,
    children_query: Query<&Children>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    let key_pressed = input.any_just_pressed([KeyCode::Escape, KeyCode::KeyS]);
    let event_fired = close_events.read().next().is_some();
    if !(key_pressed || event_fired) {
        return;
    }

    let Some(menu_entity) = active_menus.pop() else {
        return;
    };

    if let Ok(children) = children_query.get(menu_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    if active_menus.is_empty() {
        next_state.set(PlayerState::World);
    }
}

#[derive(Event)]
pub struct ClosedMenu<T>(PhantomData<T>);

fn close_menu_trigger<T: Clone + Send + Sync + 'static>(
    mut close_events: EventReader<ClosedMenu<T>>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let key_pressed = input.any_just_pressed([KeyCode::Escape, KeyCode::KeyS]);
    let event_fired = close_events.read().next().is_some();
    if !(key_pressed || event_fired) {
        return;
    }

    commands.trigger(ClosedMenu::<T>(PhantomData));
}

#[derive(Resource, Deref, DerefMut, Default)]
struct ActiveMenus(Vec<Entity>);

#[derive(Event)]
pub struct SelectionEvent<T> {
    pub selection: T,
    pub menu: Entity,
    pub entry: Entity,
}

fn selection_callback<T: Clone + Send + Sync + 'static>(
    mut commands: Commands,
    active: Res<ActiveMenus>,
    children: Query<&Children>,
    menu_query: Query<&NodePayload<T>, With<Selected>>,
) {
    let Some(menu) = active.last() else {
        return;
    };
    let Ok(children) = children.get(*menu) else {
        return;
    };

    for child in children.iter() {
        let Ok(selected) = menu_query.get(child) else {
            continue;
        };

        commands.trigger(SelectionEvent {
            selection: (**selected).clone(),
            menu: *menu,
            entry: child,
        });
    }
}
